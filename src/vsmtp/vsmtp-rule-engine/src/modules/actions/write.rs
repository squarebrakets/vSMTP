/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
*/
use rhai::plugin::{
    mem, Dynamic, EvalAltResult, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};

#[rhai::plugin::export_module]
pub mod write {

    use crate::modules::types::types::{Context, Server};
    use crate::{modules::mail_context::mail_context::message_id, modules::EngineResult};
    use vsmtp_common::mail_context::MessageBody;
    use vsmtp_config::create_app_folder;

    /// write the current email to a specified folder.
    #[rhai_fn(global, return_raw, pure)]
    pub fn write(srv: &mut Server, mut ctx: Context, dir: &str) -> EngineResult<()> {
        let mut dir =
            create_app_folder(&srv.config, Some(dir)).map_err::<Box<EvalAltResult>, _>(|err| {
                format!(
                    "failed to write email at {}/{dir}: {err}",
                    srv.config.app.dirpath.display()
                )
                .into()
            })?;
        dir.push(format!("{}.eml", message_id(&mut ctx)?));

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&dir)
            .map_err::<Box<EvalAltResult>, _>(|err| {
                format!("failed to write email at {dir:?}: {err}").into()
            })?;
        let mut writer = std::io::LineWriter::new(file);

        let body = &ctx
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .body;
        if MessageBody::Empty == *body {
            return Err("failed to write email: the body has not been received yet.".into());
        }

        std::io::Write::write_all(&mut writer, body.to_string().as_bytes())
            .map_err(|err| format!("failed to write email at {dir:?}: {err}").into())
    }

    /// write the content of the current email with it's metadata in a json file.
    #[rhai_fn(global, return_raw, pure)]
    pub fn dump(srv: &mut Server, mut ctx: Context, dir: &str) -> EngineResult<()> {
        let mut dir =
            create_app_folder(&srv.config, Some(dir)).map_err::<Box<EvalAltResult>, _>(|err| {
                format!(
                    "failed to dump email at {}/{dir}: {err}",
                    srv.config.app.dirpath.display()
                )
                .into()
            })?;

        dir.push(format!("{}.json", message_id(&mut ctx)?));

        match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&dir)
        {
            Ok(mut file) => std::io::Write::write_all(
                &mut file,
                vsmtp_common::re::serde_json::to_string_pretty(
                    &*ctx
                        .read()
                        .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?,
                )
                .map_err::<Box<EvalAltResult>, _>(|err| {
                    format!("failed to dump email at {dir:?}: {err}").into()
                })?
                .as_bytes(),
            )
            .map_err(|err| format!("failed to dump email at {dir:?}: {err}").into()),
            Err(err) => Err(format!("failed to dump email at {dir:?}: {err}").into()),
        }
    }
}