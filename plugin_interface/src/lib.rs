///通用接口，暂时只设计execute
pub trait PluginService {
    fn execute(&self, option: &str);
}
