use aes::cipher::KeyIvInit;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use pumpkin_protocol::StreamEncryptor;
use std::hint::black_box;
use std::io::Error;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::task::{Context, Poll};
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

const KEY: [u8; 16] = [0x5Au8; 16];

struct CountingWriter<W> {
    writer: W,
    writes: Arc<AtomicUsize>,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for CountingWriter<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        if !buf.is_empty() {
            self.writes.fetch_add(1, Ordering::Relaxed);
        }
        Pin::new(&mut self.writer).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.writer).poll_shutdown(cx)
    }
}

struct BytewiseEncryptor<W: AsyncWrite + Unpin> {
    cipher: cfb8::Encryptor<aes::Aes128>,
    writer: W,
}

impl<W: AsyncWrite + Unpin> BytewiseEncryptor<W> {
    const fn new(cipher: cfb8::Encryptor<aes::Aes128>, writer: W) -> Self {
        Self { cipher, writer }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for BytewiseEncryptor<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let mut ciphertext = [buf[0]];
        self.cipher.encrypt(&mut ciphertext);
        match Pin::new(&mut self.writer).poll_write(cx, &ciphertext) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(0)) => Poll::Ready(Err(Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write encrypted data",
            ))),
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(1)),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.writer).poll_shutdown(cx)
    }
}

async fn encrypt_buffered(payload: &[u8], fragment_size: usize) -> usize {
    let writes = Arc::new(AtomicUsize::new(0));
    let cipher = cfb8::Encryptor::<aes::Aes128>::new_from_slices(&KEY, &KEY).unwrap();
    let sink = CountingWriter {
        writer: tokio::io::sink(),
        writes: writes.clone(),
    };
    let mut writer = StreamEncryptor::new(cipher, sink);
    for fragment in payload.chunks(fragment_size) {
        writer.write_all(fragment).await.unwrap();
    }
    writer.shutdown().await.unwrap();
    writes.load(Ordering::Relaxed)
}

async fn encrypt_bytewise(payload: &[u8], fragment_size: usize) -> usize {
    let writes = Arc::new(AtomicUsize::new(0));
    let cipher = cfb8::Encryptor::<aes::Aes128>::new_from_slices(&KEY, &KEY).unwrap();
    let sink = BufWriter::new(CountingWriter {
        writer: tokio::io::sink(),
        writes: writes.clone(),
    });
    let mut writer = BytewiseEncryptor::new(cipher, sink);
    for fragment in payload.chunks(fragment_size) {
        writer.write_all(fragment).await.unwrap();
    }
    writer.shutdown().await.unwrap();
    writes.load(Ordering::Relaxed)
}

fn bench_stream_encryption(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("stream_encryption");

    for size in [64, 1024, 16 * 1024, 64 * 1024, 1024 * 1024] {
        let payload: Vec<u8> = (0..size).map(|index| (index % 251) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("old_bytewise_bufwriter", size),
            &payload,
            |b, payload| {
                b.to_async(&runtime).iter(|| async {
                    let writes =
                        black_box(encrypt_bytewise(black_box(payload), payload.len()).await);
                    let expected = payload.len().div_ceil(8 * 1024);
                    assert!((expected..=expected + 1).contains(&writes));
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("buffered_direct", size),
            &payload,
            |b, payload| {
                b.to_async(&runtime).iter(|| async {
                    let writes =
                        black_box(encrypt_buffered(black_box(payload), payload.len()).await);
                    let expected = payload.len().div_ceil(16 * 1024);
                    assert!((expected..=expected + 1).contains(&writes));
                });
            },
        );
    }
    group.finish();

    let payload: Vec<u8> = (0..64 * 1024).map(|index| (index % 251) as u8).collect();
    let mut group = c.benchmark_group("stream_encryption_fragmented_64k");
    group.throughput(Throughput::Bytes(payload.len() as u64));

    for fragment_size in [64, 1024, 16 * 1024] {
        group.bench_with_input(
            BenchmarkId::new("old_bytewise_bufwriter", fragment_size),
            &fragment_size,
            |b, &fragment_size| {
                b.to_async(&runtime).iter(|| async {
                    black_box(
                        encrypt_bytewise(black_box(&payload), black_box(fragment_size)).await,
                    );
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("buffered_direct", fragment_size),
            &fragment_size,
            |b, &fragment_size| {
                b.to_async(&runtime).iter(|| async {
                    black_box(
                        encrypt_buffered(black_box(&payload), black_box(fragment_size)).await,
                    );
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_stream_encryption);
criterion_main!(benches);
