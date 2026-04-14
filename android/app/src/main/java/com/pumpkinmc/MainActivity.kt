package com.pumpkinmc

import android.content.ClipboardManager
import android.os.Bundle
import android.text.InputType
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.*
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import com.pumpkinmc.ui.theme.PumpkinmcTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

private val ANSI_REGEX = Regex("\u001b\\[([0-9;?<=>]*)([a-zA-Z])")
private const val MAX_LINES = 1000

class MainActivity : ComponentActivity() {
    private external fun openPty(command: String, cwd: String): Int
    private external fun readPty(fd: Int): String?
    private external fun writePty(fd: Int, text: String)
    private external fun closePty(fd: Int)

    companion object {
        init { System.loadLibrary("rust_terminal") }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            PumpkinmcTheme {
                TerminalScreen(::openPty, ::readPty, ::writePty, ::closePty)
            }
        }
    }
}

@Composable
fun TerminalScreen(
    onOpenPty: (String, String) -> Int,
    onReadPty: (Int) -> String?,
    onWritePty: (Int, String) -> Unit,
    onClosePty: (Int) -> Unit,
) {
    val context = LocalContext.current
    val completedLines = remember {
        mutableStateListOf(buildAnnotatedString { append("Initializing pumpkin server with TTY...") })
    }
    var currentLineRaw by remember { mutableStateOf("") }
    val coroutineScope = rememberCoroutineScope()
    val listState = rememberLazyListState()
    val ptyFd = remember { mutableStateOf<Int?>(null) }
    val terminalViewRef = remember { arrayOfNulls<View>(1) }

    val inputHandler = remember { object { var send: (String) -> Unit = {} } }

    var cursorVisible by remember { mutableStateOf(true) }
    LaunchedEffect(Unit) {
        while (true) { delay(500); cursorVisible = !cursorVisible }
    }
    
    val cursorRef = remember { intArrayOf(0) }

    fun appendText(text: String) {
        val sb = StringBuilder(currentLineRaw)
        var cursor = cursorRef[0]
        var i = 0

        fun visualToRaw(col: Int): Int {
            var raw = 0
            var vis = 0
            while (raw < sb.length) {
                while (raw < sb.length && sb[raw] == '\u001b' && raw + 1 < sb.length && sb[raw + 1] == '[') {
                    var k = raw + 2
                    while (k < sb.length && (sb[k] in '0'..'9' || sb[k] == ';' || sb[k] == '?' || sb[k] == '<' || sb[k] == '=' || sb[k] == '>')) k++
                    raw = if (k < sb.length) k + 1 else k
                }
                if (vis >= col) return raw
                if (raw < sb.length) { raw++; vis++ }
            }
            return sb.length
        }

        fun rawToVisual(rawPos: Int): Int {
            var raw = 0
            var vis = 0
            val limit = rawPos.coerceAtMost(sb.length)
            while (raw < limit) {
                if (sb[raw] == '\u001b' && raw + 1 < sb.length && sb[raw + 1] == '[') {
                    var k = raw + 2
                    while (k < sb.length && (sb[k] in '0'..'9' || sb[k] == ';'
                                || sb[k] == '?' || sb[k] == '<' || sb[k] == '=' || sb[k] == '>')) k++
                    raw = if (k < sb.length) k + 1 else k
                } else { raw++; vis++ }
            }
            return vis
        }

        while (i < text.length) {
            val ch = text[i]
            if (ch == '\u001b') {
                if (i + 1 >= text.length) { i++; continue }
                when (text[i + 1]) {
                    '[' -> {
                        var j = i + 2
                        while (j < text.length && (text[j] in '0'..'9' || text[j] == ';'
                                    || text[j] == '?' || text[j] == '<' || text[j] == '=' || text[j] == '>')) j++
                        if (j < text.length) {
                            val params = text.substring(i + 2, j)
                            when (text[j]) {
                                'K' -> when (params.toIntOrNull() ?: 0) {
                                    0 -> sb.delete(cursor, sb.length)
                                    1 -> { sb.delete(0, cursor); cursor = 0 }
                                    2 -> { sb.clear(); cursor = 0 }
                                }
                                'J' -> if ((params.toIntOrNull() ?: 0) >= 2) {
                                    completedLines.clear(); sb.clear(); cursor = 0
                                }
                                'm' -> {
                                    val code = text.substring(i, j + 1)
                                    sb.insert(cursor, code)
                                    cursor += code.length
                                }
                                'G' -> {
                                    val col = (params.toIntOrNull() ?: 1).coerceAtLeast(1)
                                    cursor = visualToRaw(col - 1)
                                }
                                'D' -> {
                                    val vis = rawToVisual(cursor)
                                    cursor = visualToRaw((vis - (params.toIntOrNull() ?: 1)).coerceAtLeast(0))
                                }
                                'C' -> {
                                    val vis = rawToVisual(cursor)
                                    val maxVis = rawToVisual(sb.length)
                                    cursor = visualToRaw((vis + (params.toIntOrNull() ?: 1)).coerceAtMost(maxVis))
                                }
                            }
                            i = j + 1
                        } else {
                            i = j
                        }
                        continue
                    }
                    ']' -> {
                        var j = i + 2
                        var found = false
                        while (j < text.length) {
                            when {
                                text[j] == '\u0007' -> { i = j + 1; found = true; break }
                                text[j] == '\u001b' && j + 1 < text.length && text[j + 1] == '\\' -> {
                                    i = j + 2; found = true; break
                                }
                                else -> j++
                            }
                        }
                        if (!found) i = text.length
                        continue
                    }
                    else -> { i += 2; continue }
                }
            }
            when (ch) {
                '\n' -> {
                    completedLines.add(parseAnsi(sb.toString()))
                    if (completedLines.size > MAX_LINES)
                        completedLines.removeRange(0, completedLines.size - MAX_LINES)
                    sb.clear()
                    cursor = 0
                }
                '\r' -> if (i + 1 >= text.length || text[i + 1] != '\n') cursor = 0
                '\b' -> if (cursor > 0) { cursor--; sb.deleteCharAt(cursor) }
                '\u007f' -> if (cursor > 0) { cursor--; sb.deleteCharAt(cursor) }
                else -> if (ch >= ' ') {
                    if (cursor < sb.length) sb.setCharAt(cursor, ch) else sb.append(ch)
                    cursor++
                }
            }
            i++
        }
        cursorRef[0] = cursor
        currentLineRaw = sb.toString()
    }

    fun sendRaw(text: String) {
        val fd = ptyFd.value ?: return
        coroutineScope.launch(Dispatchers.IO) { onWritePty(fd, text) }
    }

    inputHandler.send = { sendRaw(it) }

    DisposableEffect(Unit) {
        val job = coroutineScope.launch(Dispatchers.IO) {
            var fd = -1
            try {
                val nativeDir = context.applicationInfo.nativeLibraryDir
                val binaryFile = File(nativeDir, "libpumpkin.so")
                if (!binaryFile.exists()) {
                    withContext(Dispatchers.Main) { appendText("\nError: Binary not found at ${binaryFile.absolutePath}") }
                    return@launch
                }
                val dataDir = context.getExternalFilesDir(null)?.parentFile
                    ?: context.filesDir
                dataDir.mkdirs()
                fd = onOpenPty(binaryFile.absolutePath, dataDir.absolutePath)
                if (fd < 0) {
                    withContext(Dispatchers.Main) { appendText("\nError: Failed to open PTY") }
                    return@launch
                }
                withContext(Dispatchers.Main) { ptyFd.value = fd }
                while (true) {
                    val text = onReadPty(fd) ?: run { delay(5); null } ?: continue
                    withContext(Dispatchers.Main) { appendText(text) }
                    if (text.contains("[Server stopped]")) break
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) { appendText("\nError: ${e.message}") }
            } finally {
                if (fd >= 0) onClosePty(fd)
            }
        }
        onDispose { job.cancel() }
    }

    LaunchedEffect(completedLines.size) {
        listState.scrollToItem(completedLines.size)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .statusBarsPadding()
            .navigationBarsPadding()
            .imePadding()
            .padding(8.dp)
    ) {
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .pointerInput(Unit) {
                    awaitEachGesture {
                        val event = awaitPointerEvent(PointerEventPass.Initial)
                        if (event.changes.any { it.pressed }) {
                            terminalViewRef[0]?.post {
                                terminalViewRef[0]?.requestFocus()
                                (terminalViewRef[0]?.context
                                    ?.getSystemService(android.content.Context.INPUT_METHOD_SERVICE) as? InputMethodManager)
                                    ?.showSoftInput(terminalViewRef[0], InputMethodManager.SHOW_IMPLICIT)
                            }
                        }
                    }
                }
        ) {
            SelectionContainer {
                LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
                    items(completedLines) { line ->
                        Text(text = line, fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                    }
                    item {
                        Text(
                            text = parseAnsi(currentLineRaw + if (cursorVisible) "█" else " "),
                            fontFamily = FontFamily.Monospace,
                            fontSize = 12.sp,
                        )
                    }
                }
            }
        }

        // Invisible focusable view that owns the IME connection.
        // Keyboard input flows through InputConnection — no ZWS tricks, no composing issues.
        AndroidView(
            factory = { ctx ->
                object : View(ctx) {
                    override fun onCheckIsTextEditor() = true

                    override fun onCreateInputConnection(outAttrs: EditorInfo): BaseInputConnection {
                        outAttrs.inputType = InputType.TYPE_NULL
                        outAttrs.imeOptions = EditorInfo.IME_FLAG_NO_EXTRACT_UI or
                                EditorInfo.IME_FLAG_NO_FULLSCREEN

                        return object : BaseInputConnection(this, false) {
                            private var composing = ""

                            override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
                                val str = text?.toString() ?: return true
                                if (str == composing) {
                                    composing = ""
                                    return true
                                }
                                repeat(composing.length) { inputHandler.send("\u007f") }
                                composing = ""
                                if (str.isNotEmpty()) inputHandler.send(str)
                                return true
                            }

                            override fun setComposingRegion(start: Int, end: Int): Boolean { composing = ""; return true }
                            override fun finishComposingText(): Boolean { composing = ""; return true }

                            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
                                composing = ""
                                repeat(beforeLength) { inputHandler.send("\u007f") }
                                return true
                            }

                            override fun sendKeyEvent(event: KeyEvent?): Boolean {
                                if (event == null || event.action != KeyEvent.ACTION_DOWN) return true
                                when (event.keyCode) {
                                    KeyEvent.KEYCODE_ENTER -> inputHandler.send("\r")
                                    KeyEvent.KEYCODE_DEL -> inputHandler.send("\u007f")
                                    KeyEvent.KEYCODE_FORWARD_DEL -> inputHandler.send("\u001b[3~")
                                    KeyEvent.KEYCODE_TAB -> inputHandler.send("\t")
                                    KeyEvent.KEYCODE_ESCAPE -> inputHandler.send("\u001b")
                                    KeyEvent.KEYCODE_DPAD_UP -> inputHandler.send("\u001b[A")
                                    KeyEvent.KEYCODE_DPAD_DOWN -> inputHandler.send("\u001b[B")
                                    KeyEvent.KEYCODE_DPAD_RIGHT -> inputHandler.send("\u001b[C")
                                    KeyEvent.KEYCODE_DPAD_LEFT -> inputHandler.send("\u001b[D")
                                    else -> {
                                        val unicode = event.unicodeChar
                                        if (unicode != 0) inputHandler.send(unicode.toChar().toString())
                                    }
                                }
                                return true
                            }

                            override fun performContextMenuAction(id: Int): Boolean {
                                if (id == android.R.id.paste) {
                                    val cm = ctx.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as? ClipboardManager
                                    val text = cm?.primaryClip?.getItemAt(0)?.coerceToText(ctx)?.toString()
                                    if (!text.isNullOrEmpty()) inputHandler.send(text)
                                    return true
                                }
                                return super.performContextMenuAction(id)
                            }
                        }
                    }
                }.apply {
                    isFocusable = true
                    isFocusableInTouchMode = true
                    post { requestFocus() }
                }.also { terminalViewRef[0] = it }
            },
            modifier = Modifier.size(1.dp)
        )
    }
}

fun parseAnsi(text: String): AnnotatedString {
    return buildAnnotatedString {
        var lastMatchEnd = 0
        var fgColor = Color.White
        var bgColor = Color.Unspecified
        var bold = false
        var italic = false
        var underline = false
        var strikethrough = false

        fun currentStyle() = SpanStyle(
            color = fgColor,
            background = bgColor,
            fontWeight = if (bold) FontWeight.Bold else null,
            fontStyle = if (italic) FontStyle.Italic else FontStyle.Normal,
            textDecoration = when {
                underline && strikethrough -> TextDecoration.combine(
                    listOf(TextDecoration.Underline, TextDecoration.LineThrough)
                )
                underline -> TextDecoration.Underline
                strikethrough -> TextDecoration.LineThrough
                else -> null
            }
        )

        fun resetAll() {
            fgColor = Color.White; bgColor = Color.Unspecified
            bold = false; italic = false; underline = false; strikethrough = false
        }

        ANSI_REGEX.findAll(text).forEach { match ->
            val before = text.substring(lastMatchEnd, match.range.first)
            if (before.isNotEmpty()) withStyle(currentStyle()) { append(before) }

            if (match.groupValues[2] == "m") {
                val codes = match.groupValues[1].ifEmpty { "0" }.split(";")
                var idx = 0
                while (idx < codes.size) {
                    when (codes[idx]) {
                        "", "0" -> resetAll()
                        "1" -> bold = true
                        "2" -> {}
                        "3" -> italic = true
                        "4", "21" -> underline = true
                        "5", "6" -> {}
                        "7" -> {
                            val tmp = fgColor
                            fgColor = if (bgColor == Color.Unspecified) Color.Black else bgColor
                            bgColor = tmp
                        }
                        "9" -> strikethrough = true
                        "22" -> bold = false
                        "23" -> italic = false
                        "24" -> underline = false
                        "25", "27" -> {}
                        "29" -> strikethrough = false
                        "30" -> fgColor = ansi256Color(0)
                        "31" -> fgColor = ansi256Color(1)
                        "32" -> fgColor = ansi256Color(2)
                        "33" -> fgColor = ansi256Color(3)
                        "34" -> fgColor = ansi256Color(4)
                        "35" -> fgColor = ansi256Color(5)
                        "36" -> fgColor = ansi256Color(6)
                        "37" -> fgColor = ansi256Color(7)
                        "38" -> if (idx + 1 < codes.size) when (codes[idx + 1]) {
                            "5" -> if (idx + 2 < codes.size) {
                                fgColor = ansi256Color(codes[idx + 2].toIntOrNull() ?: 0); idx += 2
                            }
                            "2" -> if (idx + 4 < codes.size) {
                                fgColor = Color(
                                    codes[idx + 2].toIntOrNull() ?: 0,
                                    codes[idx + 3].toIntOrNull() ?: 0,
                                    codes[idx + 4].toIntOrNull() ?: 0
                                ); idx += 4
                            }
                        }
                        "39" -> fgColor = Color.White
                        "40" -> bgColor = ansi256Color(0)
                        "41" -> bgColor = ansi256Color(1)
                        "42" -> bgColor = ansi256Color(2)
                        "43" -> bgColor = ansi256Color(3)
                        "44" -> bgColor = ansi256Color(4)
                        "45" -> bgColor = ansi256Color(5)
                        "46" -> bgColor = ansi256Color(6)
                        "47" -> bgColor = ansi256Color(7)
                        "48" -> if (idx + 1 < codes.size) when (codes[idx + 1]) {
                            "5" -> if (idx + 2 < codes.size) {
                                bgColor = ansi256Color(codes[idx + 2].toIntOrNull() ?: 0); idx += 2
                            }
                            "2" -> if (idx + 4 < codes.size) {
                                bgColor = Color(
                                    codes[idx + 2].toIntOrNull() ?: 0,
                                    codes[idx + 3].toIntOrNull() ?: 0,
                                    codes[idx + 4].toIntOrNull() ?: 0
                                ); idx += 4
                            }
                        }
                        "49" -> bgColor = Color.Unspecified
                        "90" -> fgColor = ansi256Color(8)
                        "91" -> fgColor = ansi256Color(9)
                        "92" -> fgColor = ansi256Color(10)
                        "93" -> fgColor = ansi256Color(11)
                        "94" -> fgColor = ansi256Color(12)
                        "95" -> fgColor = ansi256Color(13)
                        "96" -> fgColor = ansi256Color(14)
                        "97" -> fgColor = ansi256Color(15)
                        "100" -> bgColor = ansi256Color(8)
                        "101" -> bgColor = ansi256Color(9)
                        "102" -> bgColor = ansi256Color(10)
                        "103" -> bgColor = ansi256Color(11)
                        "104" -> bgColor = ansi256Color(12)
                        "105" -> bgColor = ansi256Color(13)
                        "106" -> bgColor = ansi256Color(14)
                        "107" -> bgColor = ansi256Color(15)
                    }
                    idx++
                }
            }
            lastMatchEnd = match.range.last + 1
        }
        val remaining = text.substring(lastMatchEnd)
        if (remaining.isNotEmpty()) withStyle(currentStyle()) { append(remaining) }
    }
}

fun ansi256Color(n: Int): Color = when {
    n < 16 -> {
        val palette = intArrayOf(
            0xFF242424.toInt(), 0xFFF62B5A.toInt(), 0xFF47B413.toInt(), 0xFFE3C401.toInt(),
            0xFF24ACD4.toInt(), 0xFFF2AFFD.toInt(), 0xFF13C299.toInt(), 0xFFE6E6E6.toInt(),
            0xFF616161.toInt(), 0xFFFF4D51.toInt(), 0xFF35D450.toInt(), 0xFFE9E836.toInt(),
            0xFF5DC5F8.toInt(), 0xFFFEABF2.toInt(), 0xFF24DFC4.toInt(), 0xFFFFFFFF.toInt()
        )
        Color(palette[n])
    }
    n < 232 -> {
        val i = n - 16
        val r = i / 36
        val g = (i / 6) % 6
        val b = i % 6
        val ramp = intArrayOf(0, 95, 135, 175, 215, 255)
        Color(red = ramp[r], green = ramp[g], blue = ramp[b])
    }
    else -> {
        val gray = (n - 232) * 10 + 8
        Color(red = gray, green = gray, blue = gray)
    }
}
