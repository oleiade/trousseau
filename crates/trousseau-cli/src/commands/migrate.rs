//! `trousseau migrate` (3.5.15).
//!
//! Reuses `init`'s target-creation logic (3.5.2) through
//! [`crate::commands::init::lock_new_target`] and
//! [`crate::commands::init::resolve_target`]: a `migrate` target follows
//! the exact same "must not already exist", recipients-or-passphrase
//! rules as a fresh `init` target, so `migrate` accepts the same
//! `--recipient`, `--recipients-file`, `--passphrase`, `--no-self`
//! flags (3.5.15).

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::Serialize;

use trousseau::legacy::{self, GpgOptions, LegacyAlgorithm};

use crate::cli::MigrateArgs;
use crate::commands::init;
use crate::context::{Context, SealMaterial};
use crate::output::{self, OutputMode};

/// Run `migrate`.
///
/// The target store is resolved and locked first (3.5.2, 3.5.15), so a
/// target that already exists fails fast, before `SOURCE` is even read.
/// `SOURCE` is only ever read, never written or deleted.
///
/// # Errors
///
/// Returns a generic error (exit 1, `not a v0.4 store`) if `SOURCE` does
/// not parse as a legacy v0.4 envelope,
/// [`crate::exit::CliError::StoreExists`] (exit 8) if the target store
/// already exists, [`crate::exit::CliError::Usage`] if `--no-self` leaves
/// no recipient, or whatever reading `SOURCE`, decrypting it (AES
/// passphrase resolution, or spawning `gpg`), resolving the target's
/// recipients or new passphrase, or [`trousseau::store::save`] returns.
pub fn run(ctx: &Context, args: &MigrateArgs) -> anyhow::Result<()> {
    let resolved = ctx.resolve_store_for_init();
    let path = resolved.path().to_path_buf();
    let _lock = init::lock_new_target(ctx, &path)?;

    let source_bytes = std::fs::read(&args.source)
        .with_context(|| format!("reading {}", args.source.display()))?;
    let envelope = legacy::parse_envelope(&source_bytes)
        .map_err(|_err| anyhow::anyhow!("not a v0.4 store"))?;

    let legacy_store = match envelope.algorithm {
        LegacyAlgorithm::Aes256Cfb => {
            let passphrase = ctx.legacy_passphrase()?;
            legacy::decrypt_aes(&envelope, &passphrase)?
        }
        LegacyAlgorithm::OpenPgp => {
            let binary = args
                .gpg
                .clone()
                .unwrap_or_else(|| PathBuf::from(&ctx.config.migrate.gpg));
            let opts = GpgOptions {
                binary,
                gnupg_home: args.gnupg_home.clone(),
            };
            legacy::decrypt_gpg(&envelope, &opts)?
        }
    };

    let now = ctx.now();
    let (kind, recipients) = init::resolve_target(ctx, &args.target)?;
    let conversion = legacy::convert(legacy_store, kind, recipients, now);

    let seal_material = if args.target.passphrase {
        SealMaterial::Passphrase(ctx.new_migrated_passphrase(args.new_passphrase_file.as_deref())?)
    } else {
        ctx.seal_for(&conversion.store)?
    };
    trousseau::store::save(&path, &conversion.store, seal_material.as_seal())?;

    report(ctx, &path, &conversion)
}

/// One renamed key, `{"from", "to"}` (appendix 5.1's `migrate` shape).
#[derive(Serialize)]
struct RenamedJson<'a> {
    from: &'a str,
    to: &'a str,
}

/// The `migrate` output shape for `--json` (appendix 5.1).
#[derive(Serialize)]
struct MigrateJson<'a> {
    ok: bool,
    path: String,
    entries: usize,
    renamed: Vec<RenamedJson<'a>>,
    legacy_recipients: &'a [String],
}

/// Print `migrate`'s result: every renamed key (`renamed "<old>" ->
/// <new>`), the legacy recipients for information, and `migrated N
/// entries to <path>` on stderr in human mode, or the JSON shape from
/// appendix 5.1 on stdout in `--json` mode.
fn report(ctx: &Context, path: &Path, conversion: &legacy::Conversion) -> anyhow::Result<()> {
    match ctx.output {
        OutputMode::Human => {
            for (from, to) in &conversion.renamed {
                output::info(ctx.quiet, &format!("renamed \"{from}\" -> {to}"));
            }
            if !conversion.legacy_recipients.is_empty() {
                output::info(
                    ctx.quiet,
                    &format!(
                        "legacy recipients: {}",
                        conversion.legacy_recipients.join(", ")
                    ),
                );
            }
            output::info(
                ctx.quiet,
                &format!(
                    "migrated {} entries to {}",
                    conversion.store.entries.len(),
                    path.display()
                ),
            );
            Ok(())
        }
        OutputMode::Json => output::json(&MigrateJson {
            ok: true,
            path: path.display().to_string(),
            entries: conversion.store.entries.len(),
            renamed: conversion
                .renamed
                .iter()
                .map(|(from, to)| RenamedJson {
                    from,
                    to: to.as_str(),
                })
                .collect(),
            legacy_recipients: &conversion.legacy_recipients,
        }),
    }
}
