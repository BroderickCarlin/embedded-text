//! Plugins allow changing TextBox behaviour.
//!
//! Note: Custom plugins are experimental. If you wish to implement custom plugins,
//! you need to activate the `plugin` feature.

use core::{
    cell::UnsafeCell,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::PixelColor,
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
};

use crate::{
    parser::{Parser, Token},
    rendering::{cursor::Cursor, TextBoxProperties},
};

#[cfg(feature = "plugin")]
pub mod private;
#[cfg(feature = "plugin")]
pub use private::Plugin;

#[cfg(not(feature = "plugin"))]
mod private;
#[cfg(not(feature = "plugin"))]
use private::Plugin;

#[cfg(feature = "ansi")]
pub mod ansi;
pub mod tail;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ProcessingState {
    Measure,
    Render,
}

/// Placeholder type when no plugin is used.
#[derive(Clone, Copy, Default)]
pub struct NoPlugin<C>(PhantomData<C>)
where
    C: PixelColor;

impl<C> NoPlugin<C>
where
    C: PixelColor,
{
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Plugin marker trait.
///
/// This trait is an implementation detail. Most likely you don't need to implement this.
#[cfg_attr(
    feature = "plugin",
    doc = "If you wish to implement a plugin, see [Plugin]."
)]
// TODO: remove this trait once Plugin is stabilized, then move Plugin here
pub trait PluginMarker<'a, C: PixelColor>: Plugin<'a, C> {}

impl<'a, C, T> PluginMarker<'a, C> for T
where
    T: Plugin<'a, C>,
    C: PixelColor,
{
}

#[derive(Clone, Debug)]
pub(crate) struct PluginInner<'a, M, C> {
    pub(crate) plugin: M,
    state: ProcessingState,
    peeked_token: Option<Token<'a, C>>,
}

#[derive(Debug)]
pub(crate) struct PluginWrapper<'a, M, C> {
    inner: UnsafeCell<PluginInner<'a, M, C>>,
}

impl<'a, M: Clone, C: Clone> Clone for PluginWrapper<'a, M, C> {
    fn clone(&self) -> Self {
        Self {
            inner: UnsafeCell::new(self.inner(|this| this.clone())),
        }
    }
}

impl<'a, M, C> Hash for PluginWrapper<'a, M, C>
where
    C: PixelColor,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner(|this| this.state.hash(state))
    }
}

impl<'a, M, C> PluginWrapper<'a, M, C> {
    pub fn new(plugin: M) -> Self {
        Self {
            inner: UnsafeCell::new(PluginInner {
                plugin,
                state: ProcessingState::Measure,
                peeked_token: None,
            }),
        }
    }

    pub fn into_inner(self) -> M {
        self.inner.into_inner().plugin
    }

    fn inner<R>(&self, cb: impl FnOnce(&mut PluginInner<'a, M, C>) -> R) -> R {
        let inner = unsafe {
            // SAFETY: This is safe because we aren't exposing the reference.
            core::ptr::NonNull::new_unchecked(self.inner.get()).as_mut()
        };

        cb(inner)
    }
}

impl<'a, M, C> PluginWrapper<'a, M, C>
where
    C: PixelColor,
    M: private::Plugin<'a, C>,
{
    pub fn new_line(&self) {
        self.inner(|this| this.plugin.new_line());
    }

    pub fn set_state(&self, state: ProcessingState) {
        self.inner(|this| this.state = state);
    }

    #[inline]
    pub fn render_token(&self, token: Token<'a, C>) -> Option<Token<'a, C>> {
        self.inner(|this| match this.state {
            ProcessingState::Measure => Some(token),
            ProcessingState::Render => this.plugin.render_token(token),
        })
    }

    pub fn peek_token(&self, source: &mut Parser<'a, C>) -> Option<Token<'a, C>> {
        self.inner(|this| {
            if this.peeked_token.is_none() {
                this.peeked_token = this.plugin.next_token(|| source.next());
            }

            this.peeked_token.clone()
        })
    }

    pub fn consume_peeked_token(&self) {
        self.inner(|this| {
            if this.peeked_token.is_some() {
                this.peeked_token = None;
            }
        });
    }

    pub fn consume_partial(&self, len: usize) {
        self.inner(|this| {
            // Only string-like tokens can be partially consumed.
            debug_assert!(matches!(
                this.peeked_token,
                Some(Token::Whitespace(_, _)) | Some(Token::Word(_))
            ));

            let skip_chars = |str: &'a str, n| {
                let mut chars = str.chars();
                for _ in 0..n {
                    chars.next();
                }
                chars.as_str()
            };

            let token = match this.peeked_token.take().unwrap() {
                Token::Whitespace(count, seq) => {
                    Token::Whitespace(count - len as u32, skip_chars(seq, len))
                }
                Token::Word(w) => Token::Word(skip_chars(w, len)),
                _ => unreachable!(),
            };

            this.peeked_token.replace(token);
        })
    }

    pub fn on_start_render<S: CharacterStyle + TextRenderer>(
        &self,
        cursor: &mut Cursor,
        props: TextBoxProperties<'_, S>,
    ) {
        self.inner(|this| {
            this.peeked_token = None;

            this.plugin.on_start_render(cursor, &props);
        });
    }

    pub fn on_rendering_finished(&self) {
        self.inner(|this| this.plugin.on_rendering_finished());
    }

    pub fn post_render<T, D>(
        &self,
        draw_target: &mut D,
        character_style: &T,
        text: Option<&str>,
        bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        self.inner(|this| {
            this.plugin
                .post_render(draw_target, character_style, text, bounds)
        })
    }
}
