use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{ConfigResource, PluginHostState},
    wit::v0_1::pumpkin::{self, plugin::config::ConfigPathElement},
};

/// The return value will not be an error for many invalid formats, this should
/// probably be fixed.
fn string_to_path(s: String) -> wasmtime::Result<pumpkin::plugin::config::ConfigPath> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let mut path = Vec::new();
    let mut current_string = String::new();
    // let mut in_string = false;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            // '"' => {
            //     in_string = !in_string;
            // }
            '.' => {
                if !current_string.is_empty() {
                    path.push(ConfigPathElement::Key(current_string));
                    current_string = String::new();
                }
            }
            '[' => {
                path.push(ConfigPathElement::Key(current_string));
                current_string = String::new();
                let mut numbers = String::new();
                loop {
                    let Some(c) = chars.next() else {
                        wasmtime::bail!("Invalid path");
                    };
                    if c == ']' {
                        break;
                    }
                    numbers.push(c);
                }
                let index = numbers.parse::<u32>()?;
                path.push(ConfigPathElement::Index(index));
            }
            c => {
                current_string.push(c);
            }
        }
    }
    if !current_string.is_empty() {
        path.push(ConfigPathElement::Key(current_string));
    }
    Ok(path)
}

impl pumpkin::plugin::config::Host for PluginHostState {}

impl PluginHostState {
    fn get_config_res(
        &self,
        res: &Resource<pumpkin::plugin::config::Config>,
    ) -> wasmtime::Result<&ConfigResource> {
        self.resource_table
            .get::<ConfigResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl pumpkin::plugin::config::HostConfig for PluginHostState {
    async fn get(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: String,
    ) -> wasmtime::Result<Option<pumpkin::plugin::config::ConfigTree>> {
        self.path_get(res, string_to_path(key)?).await
    }

    async fn get_or_default(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: String,
        default: pumpkin::plugin::config::ConfigTree,
    ) -> wasmtime::Result<pumpkin::plugin::config::ConfigTree> {
        self.path_get_or_default(res, string_to_path(key)?, default)
            .await
    }

    async fn set(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: String,
        value: pumpkin::plugin::config::ConfigTree,
    ) -> wasmtime::Result<()> {
        self.path_set(res, string_to_path(key)?, value).await
    }

    async fn remove(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: String,
    ) -> wasmtime::Result<bool> {
        self.path_remove(res, string_to_path(key)?).await
    }

    async fn overwrite(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        config: pumpkin::plugin::config::ConfigTree,
    ) -> wasmtime::Result<()> {
        let res = self.get_config_res(&res)?;
        Ok(res.provider.set(Vec::new(), config).await?)
    }

    async fn has_key(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: String,
    ) -> wasmtime::Result<bool> {
        self.path_has_key(res, string_to_path(key)?).await
    }

    async fn path_get(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: pumpkin::plugin::config::ConfigPath,
    ) -> wasmtime::Result<Option<pumpkin::plugin::config::ConfigTree>> {
        let res = self.get_config_res(&res)?;
        Ok(res.provider.get(key).await?)
    }

    async fn path_get_or_default(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: pumpkin::plugin::config::ConfigPath,
        default: pumpkin::plugin::config::ConfigTree,
    ) -> wasmtime::Result<pumpkin::plugin::config::ConfigTree> {
        let res = self.get_config_res(&res)?;
        Ok(res.provider.get(key).await?.unwrap_or(default))
    }

    async fn path_set(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: pumpkin::plugin::config::ConfigPath,
        value: pumpkin::plugin::config::ConfigTree,
    ) -> wasmtime::Result<()> {
        let res = self.get_config_res(&res)?;
        Ok(res.provider.set(key, value).await?)
    }

    async fn path_remove(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: pumpkin::plugin::config::ConfigPath,
    ) -> wasmtime::Result<bool> {
        let res = self.get_config_res(&res)?;
        Ok(res.provider.remove(key).await?)
    }

    async fn path_has_key(
        &mut self,
        res: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
        key: pumpkin::plugin::config::ConfigPath,
    ) -> wasmtime::Result<bool> {
        let res = self.get_config_res(&res)?;
        Ok(res.provider.has_key(key).await?)
    }

    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<pumpkin::plugin::config::Config>,
    ) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ConfigResource>(wasmtime::component::Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_parsing() {
        {
            let mut path = string_to_path("a.bb.ccc".to_owned()).unwrap().into_iter();
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "a");
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "bb");
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "ccc");
            assert!(path.next().is_none());
        }

        {
            let mut path = string_to_path("xyz.bb[10].ccc".to_owned())
                .unwrap()
                .into_iter();
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "xyz");
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "bb");
            let Some(ConfigPathElement::Index(i)) = path.next() else {
                panic!();
            };
            assert_eq!(i, 10);
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "ccc");
            assert!(path.next().is_none());
        }

        assert!(string_to_path(String::new()).unwrap().is_empty());

        {
            let mut path = string_to_path("xyz[0]".to_owned()).unwrap().into_iter();
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "xyz");
            let Some(ConfigPathElement::Index(i)) = path.next() else {
                panic!();
            };
            assert_eq!(i, 0);
            assert!(path.next().is_none());
        }

        {
            let mut path = string_to_path("a[0].b".to_owned()).unwrap().into_iter();
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "a");
            let Some(ConfigPathElement::Index(i)) = path.next() else {
                panic!();
            };
            assert_eq!(i, 0);
            let Some(ConfigPathElement::Key(s)) = path.next() else {
                panic!();
            };
            assert_eq!(s, "b");
            assert!(path.next().is_none());
        }
    }
}
