use crate::*;

#[derive(Clone, Debug, PartialEq, Properties, Copy)]
pub struct MatThemeSetterProps {
    #[prop_or_default]
    pub primary: ThemePrimary,
    #[prop_or_default]
    pub secondary: ThemeSecondary,
    #[prop_or_default]
    pub surface: ThemeSurface,
    #[prop_or_default]
    pub background: ThemeBackground,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct ThemePrimary {
    pub background: Color,
    pub forground: Color,
}
impl Default for ThemePrimary {
    fn default() -> Self {
        Self {
            background: Color(0x62_00_EE_FF),
            forground: Color(0xFF_FF_FF_FF),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct ThemeSecondary {
    pub background: Color,
    pub forground: Color,
}
impl Default for ThemeSecondary {
    fn default() -> Self {
        Self {
            background: Color(0x01_87_86_FF),
            forground: Color(0xFF_FF_FF_FF),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct ThemeSurface {
    pub background: Color,
    pub forground: Color,
}
impl Default for ThemeSurface {
    fn default() -> Self {
        Self {
            background: Color(0xFF_FF_FF_FF),
            forground: Color(0x00_00_00_00),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeBackground(pub Color);

impl Default for ThemeBackground {
    fn default() -> Self {
        Self(Color(0xFF_FF_FF_FF))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u32);

impl std::fmt::UpperHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.0, f)
    }
}

impl std::fmt::UpperHex for ThemeBackground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.0, f)
    }
}

impl MatThemeSetterProps {
    pub const DARK_THEME: Self = Self {
        primary: ThemePrimary {
            background: Color(0xD0_BC_FF_FF),
            forground: Color(0x38_1E_72_FF),
        },
        secondary: ThemeSecondary {
            background: Color(0xCC_C2_DC_FF),
            forground: Color(0x33_2D_41_FF),
        },
        surface: ThemeSurface {
            background: Color(0x1C_1B_1F_FF),
            forground: Color(0xE6_E1_E5_FF),
        },
        background: ThemeBackground(Color(0x00_00_00_FF)),
    };
}

#[function_component(MatThemeSetter)]
pub fn theme(
    MatThemeSetterProps {
        primary,
        secondary,
        surface,
        background,
    }: &MatThemeSetterProps,
) -> Html {
    let ThemePrimary {
        background: p_bg,
        forground: p_fg,
    } = primary;
    let ThemeSecondary {
        background: s_bg,
        forground: s_fg,
    } = secondary;
    let ThemeSurface {
        background: su_bg,
        forground: su_fg,
    } = surface;
    let style = html! {
        <style>
            {format!(r"
                html {{
                    --mdc-theme-primary: #{p_bg:08X};
                    --mdc-theme-on-primary: #{p_fg:08X};

                    --mdc-theme-secondary: #{s_bg:08X};
                    --mdc-theme-on-secondary: #{s_fg:08X};

                    --mdc-theme-surface: #{su_bg:08X};
                    --mdc-theme-on-surface: #{su_fg:08X};

                    --mdc-theme-background: #{background:08X};

                    background-color: #{background:08X};
                    color: #{su_fg:08X};
                }}
            ")}
        </style>
    };
    yew::create_portal(style, gloo_utils::document().head().unwrap().into())
}
