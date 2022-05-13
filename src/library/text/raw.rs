use std::sync::Arc;

use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    Color, FontStyle, Highlighter, Style, StyleModifier, Theme, ThemeItem, ThemeSettings,
};
use syntect::parsing::SyntaxSet;

use super::{FontFamily, Hyphenate, TextNode, Toggle};
use crate::library::layout::BlockSpacing;
use crate::library::prelude::*;
use crate::source::SourceId;
use crate::syntax::{self, GreenNode, NodeKind, RedNode};

/// Monospaced text with optional syntax highlighting.
#[derive(Debug, Hash)]
pub struct RawNode {
    /// The raw text.
    pub text: EcoString,
    /// Whether the node is block-level.
    pub block: bool,
}

#[node(showable)]
impl RawNode {
    /// The language to syntax-highlight in.
    #[property(referenced)]
    pub const LANG: Option<EcoString> = None;

    /// The raw text's font family. Just the normal text family if `none`.
    #[property(referenced)]
    pub const FAMILY: Smart<FontFamily> = Smart::Custom(FontFamily::new("IBM Plex Mono"));

    /// The spacing above block-level raw.
    #[property(resolve, shorthand(around))]
    pub const ABOVE: Option<BlockSpacing> = Some(Ratio::one().into());
    /// The spacing below block-level raw.
    #[property(resolve, shorthand(around))]
    pub const BELOW: Option<BlockSpacing> = Some(Ratio::one().into());

    fn construct(_: &mut Context, args: &mut Args) -> TypResult<Content> {
        Ok(Content::show(Self {
            text: args.expect("text")?,
            block: args.named("block")?.unwrap_or(false),
        }))
    }
}

impl Show for RawNode {
    fn unguard(&self, _: Selector) -> ShowNode {
        Self { text: self.text.clone(), ..*self }.pack()
    }

    fn encode(&self, styles: StyleChain) -> Dict {
        dict! {
           "text" => Value::Str(self.text.clone()),
           "block" => Value::Bool(self.block),
           "lang" => match styles.get(Self::LANG) {
               Some(lang) => Value::Str(lang.clone()),
               None => Value::None,
           },
        }
    }

    fn realize(&self, _: &mut Context, styles: StyleChain) -> TypResult<Content> {
        let lang = styles.get(Self::LANG).as_ref().map(|s| s.to_lowercase());
        let foreground = THEME
            .settings
            .foreground
            .map(Color::from)
            .unwrap_or(Color::BLACK)
            .into();

        let mut realized = if matches!(lang.as_deref(), Some("typ" | "typst" | "typc")) {
            let root = match lang.as_deref() {
                Some("typc") => {
                    let children = crate::parse::parse_code(&self.text);
                    Arc::new(GreenNode::with_children(NodeKind::CodeBlock, children))
                }
                _ => crate::parse::parse(&self.text),
            };

            let red = RedNode::from_root(root, SourceId::from_raw(0));
            let highlighter = Highlighter::new(&THEME);

            let mut seq = vec![];
            syntax::highlight_syntect(red.as_ref(), &highlighter, &mut |range, style| {
                seq.push(styled(&self.text[range], foreground, style));
            });

            Content::sequence(seq)
        } else if let Some(syntax) =
            lang.and_then(|token| SYNTAXES.find_syntax_by_token(&token))
        {
            let mut seq = vec![];
            let mut highlighter = HighlightLines::new(syntax, &THEME);
            for (i, line) in self.text.lines().enumerate() {
                if i != 0 {
                    seq.push(Content::Linebreak { justified: false });
                }

                for (style, piece) in highlighter.highlight(line, &SYNTAXES) {
                    seq.push(styled(piece, foreground, style));
                }
            }

            Content::sequence(seq)
        } else {
            Content::Text(self.text.clone())
        };

        if self.block {
            realized = Content::block(realized);
        }

        Ok(realized)
    }

    fn finalize(
        &self,
        _: &mut Context,
        styles: StyleChain,
        mut realized: Content,
    ) -> TypResult<Content> {
        let mut map = StyleMap::new();
        map.set(TextNode::OVERHANG, false);
        map.set(TextNode::HYPHENATE, Smart::Custom(Hyphenate(false)));
        map.set(TextNode::SMART_QUOTES, false);

        if let Smart::Custom(family) = styles.get(Self::FAMILY) {
            map.set_family(family.clone(), styles);
        }

        if self.block {
            realized = realized.spaced(styles.get(Self::ABOVE), styles.get(Self::BELOW));
        }

        Ok(realized.styled_with_map(map))
    }
}

/// Style a piece of text with a syntect style.
fn styled(piece: &str, foreground: Paint, style: Style) -> Content {
    let mut styles = StyleMap::new();
    let mut body = Content::Text(piece.into());

    let paint = style.foreground.into();
    if paint != foreground {
        styles.set(TextNode::FILL, paint);
    }

    if style.font_style.contains(FontStyle::BOLD) {
        styles.set(TextNode::STRONG, Toggle);
    }

    if style.font_style.contains(FontStyle::ITALIC) {
        styles.set(TextNode::EMPH, Toggle);
    }

    if style.font_style.contains(FontStyle::UNDERLINE) {
        body = body.underlined();
    }

    body.styled_with_map(styles)
}

/// The lazily-loaded syntect syntax definitions.
static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());

/// The lazily-loaded theme used for syntax highlighting.
#[rustfmt::skip]
static THEME: Lazy<Theme> = Lazy::new(|| Theme {
    name: Some("Typst Light".into()),
    author: Some("The Typst Project Developers".into()),
    settings: ThemeSettings::default(),
    scopes: vec![
        item("markup.bold", None, Some(FontStyle::BOLD)),
        item("markup.italic", None, Some(FontStyle::ITALIC)),
        item("markup.heading, entity.name.section", None, Some(FontStyle::BOLD | FontStyle::UNDERLINE)),
        item("markup.raw", Some("#818181"), None),
        item("markup.list", Some("#8b41b1"), None),
        item("comment", Some("#8a8a8a"), None),
        item("keyword, constant.language, variable.language", Some("#d73a49"), None),
        item("storage.type, storage.modifier", Some("#d73a49"), None),
        item("entity.other", Some("#8b41b1"), None),
        item("entity.name, variable.function, support", Some("#4b69c6"), None),
        item("support.macro", Some("#16718d"), None),
        item("meta.annotation", Some("#301414"), None),
        item("constant", Some("#b60157"), None),
        item("string", Some("#298e0d"), None),
        item("punctuation.shortcut", Some("#1d6c76"), None),
        item("constant.character.escape", Some("#1d6c76"), None),
    ],
});

/// Create a syntect theme item.
fn item(scope: &str, color: Option<&str>, font_style: Option<FontStyle>) -> ThemeItem {
    ThemeItem {
        scope: scope.parse().unwrap(),
        style: StyleModifier {
            foreground: color.map(|s| s.parse().unwrap()),
            background: None,
            font_style,
        },
    }
}