use std::{collections::HashMap, io::Result};

use nu_protocol::{
    engine::{EngineState, Stack},
    Value,
};
use ratatui::layout::Rect;

use crate::{
    nu_common::{try_build_table, NuSpan},
    pager::Frame,
    util::map_into_value,
    views::{Layout, Preview, View, ViewConfig},
};

use super::{HelpExample, HelpManual, ViewCommand};

#[derive(Clone)]
pub struct ConfigShowCmd {
    format: ConfigFormat,
}

#[derive(Clone)]
enum ConfigFormat {
    Table,
    Nu,
}

impl ConfigShowCmd {
    pub fn new() -> Self {
        ConfigShowCmd {
            format: ConfigFormat::Table,
        }
    }
}

impl ConfigShowCmd {
    pub const NAME: &'static str = "config-show";
}

impl ViewCommand for ConfigShowCmd {
    type View = ConfigView;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn usage(&self) -> &'static str {
        ""
    }

    fn help(&self) -> Option<HelpManual> {
        Some(HelpManual {
            name: Self::NAME,
            description:
                "Show the current `explore` configuration.\nSome default fields might be missing.",
            arguments: vec![HelpExample::new("nu", "Use a nuon format instead")],
            config_options: vec![],
            input: vec![],
            examples: vec![],
        })
    }

    fn display_config_option(&mut self, _: String, _: String, _: String) -> bool {
        false
    }

    fn parse(&mut self, args: &str) -> Result<()> {
        if args.trim() == "nu" {
            self.format = ConfigFormat::Nu;
        }

        Ok(())
    }

    fn spawn(&mut self, _: &EngineState, _: &mut Stack, _: Option<Value>) -> Result<Self::View> {
        Ok(ConfigView {
            preview: Preview::new(""),
            format: self.format.clone(),
        })
    }
}

pub struct ConfigView {
    preview: Preview,
    format: ConfigFormat,
}

impl View for ConfigView {
    fn draw(&mut self, f: &mut Frame, area: Rect, cfg: ViewConfig<'_>, layout: &mut Layout) {
        self.preview.draw(f, area, cfg, layout)
    }

    fn handle_input(
        &mut self,
        engine_state: &EngineState,
        stack: &mut Stack,
        layout: &Layout,
        info: &mut crate::pager::ViewInfo,
        key: crossterm::event::KeyEvent,
    ) -> Option<crate::pager::Transition> {
        self.preview
            .handle_input(engine_state, stack, layout, info, key)
    }

    fn setup(&mut self, config: ViewConfig<'_>) {
        let text = self.create_output_string(config);

        self.preview = Preview::new(&text);
        self.preview
            .set_value(map_into_value(config.config.clone()));
    }

    fn exit(&mut self) -> Option<Value> {
        self.preview.exit()
    }

    fn collect_data(&self) -> Vec<crate::nu_common::NuText> {
        self.preview.collect_data()
    }

    fn show_data(&mut self, i: usize) -> bool {
        self.preview.show_data(i)
    }
}

impl ConfigView {
    fn create_output_string(&mut self, config: ViewConfig) -> String {
        match self.format {
            ConfigFormat::Table => {
                let mut m = config.config.clone();
                convert_styles(&mut m);

                let value = map_into_value(m);
                try_build_table(None, config.nu_config, config.style_computer, value)
            }
            ConfigFormat::Nu => nu_json::to_string(&config.config).unwrap_or_default(),
        }
    }
}

fn convert_styles(m: &mut HashMap<String, Value>) {
    for value in m.values_mut() {
        convert_styles_value(value);
    }
}

fn convert_styles_value(value: &mut Value) {
    match value {
        Value::String { val, .. } => {
            if let Some(v) = convert_style_from_string(val) {
                *value = v;
            }
        }
        Value::List { vals, .. } => {
            for value in vals {
                convert_styles_value(value);
            }
        }
        Value::Record { vals, .. } => {
            for value in vals {
                convert_styles_value(value);
            }
        }
        _ => (),
    }
}

fn convert_style_from_string(s: &str) -> Option<Value> {
    let style = nu_json::from_str::<nu_color_config::NuStyle>(s).ok()?;
    let cols = vec![String::from("bg"), String::from("fg"), String::from("attr")];

    let vals = vec![
        Value::string(style.bg.unwrap_or_default(), NuSpan::unknown()),
        Value::string(style.fg.unwrap_or_default(), NuSpan::unknown()),
        Value::string(style.attr.unwrap_or_default(), NuSpan::unknown()),
    ];

    Some(Value::Record {
        cols,
        vals,
        span: NuSpan::unknown(),
    })
}
