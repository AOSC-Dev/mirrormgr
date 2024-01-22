use eyre::Result;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester, LanguageLoader,
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::I18N_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::I18N_LOADER, $message_id, $($args), *)
    }};
}

pub static I18N_LOADER: Lazy<FluentLanguageLoader> =
    Lazy::new(|| load_i18n().expect("Unable to load i18n strings."));

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

fn load_i18n() -> Result<FluentLanguageLoader> {
    let language_loader: FluentLanguageLoader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let fallback_language: &[LanguageIdentifier] = &["en-US".parse().unwrap()];
    let languages: Vec<&LanguageIdentifier> = requested_languages
        .iter()
        .chain(fallback_language.iter())
        .collect();

    language_loader.load_languages(&Localizations, &languages)?;

    // Windows Terminal doesn't support bidirectional (BiDi) text, and renders the isolate characters incorrectly.
    // This is a temporary workaround for https://github.com/microsoft/terminal/issues/16574
    // TODO: this might break BiDi text, though we don't support any writing system depends on that.
    language_loader.set_use_isolating(false);

    Ok(language_loader)
}
