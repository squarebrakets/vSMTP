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

use vsmtp_plugins::rhai;

use crate::api::{
    EngineResult, {Message, SharedObject},
};
use rhai::plugin::{
    mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};

pub use message_rhai::*;

#[rhai::plugin::export_module]
mod message_rhai {

    /// Return a boolean, `true` if a header named `header` exists in the message.
    ///
    /// # Examples
    ///
    /// ```
    /// # let msg = vsmtp_mail_parser::MessageBody::try_from(concat!(
    /// "X-My-Header: foo\r\n",
    /// "Subject: Unit test are cool\r\n",
    /// "\r\n",
    /// "Hello world!\r\n",
    /// # )).unwrap();
    ///
    /// # let states = vsmtp_test::vsl::run_with_msg(r#"
    /// #{
    ///   preq: [
    ///     rule "check if header exists" || {
    ///       if has_header("X-My-Header") {
    ///         accept();
    ///       } else {
    ///         deny();
    ///       }
    ///     }
    ///   ]
    /// }
    /// # "#, Some(msg));
    /// # use vsmtp_common::{state::State, status::Status, CodeID};
    /// # assert_eq!(states[&State::PreQ].2, Status::Accept(either::Left(CodeID::Ok)));
    /// ```
    #[rhai_fn(global, name = "has_header", return_raw, pure)]
    pub fn has_header(message: &mut Message, header: &str) -> EngineResult<bool> {
        Ok(vsl_guard_ok!(message.read()).get_header(header).is_some())
    }

    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    #[rhai_fn(global, name = "has_header", return_raw, pure)]
    pub fn has_header_obj(message: &mut Message, header: SharedObject) -> EngineResult<bool> {
        has_header(message, &header.to_string())
    }

    /// Count the number of headers with the given name.
    ///
    /// # Examples
    ///
    /// ```
    /// # let msg = vsmtp_mail_parser::MessageBody::try_from(concat!(
    /// "X-My-Header: foo\r\n",
    /// "X-My-Header: bar\r\n",
    /// "X-My-Header: baz\r\n",
    /// "Subject: Unit test are cool\r\n",
    /// "\r\n",
    /// "Hello world!\r\n",
    /// # )).unwrap();
    ///
    /// # let states = vsmtp_test::vsl::run_with_msg(r#"
    /// #{
    ///   preq: [
    ///     rule "count_header" || {
    ///       accept(`250 count is ${count_header("X-My-Header")}`)
    ///     }
    ///   ]
    /// }
    /// # "#, Some(msg));
    /// # use vsmtp_common::{state::State, status::Status, CodeID, Reply, ReplyCode::Code};
    /// # assert_eq!(states[&State::PreQ].2, Status::Accept(either::Right(Reply::new(
    /// #  Code { code: 250 }, "count is 3".to_string(),
    /// # ))));
    /// ```
    #[rhai_fn(global, name = "count_header", return_raw, pure)]
    pub fn count_header(message: &mut Message, header: &str) -> EngineResult<rhai::INT> {
        super::count_header(message, header)
    }

    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    #[rhai_fn(global, name = "count_header", return_raw, pure)]
    pub fn count_header_obj(
        message: &mut Message,
        header: SharedObject,
    ) -> EngineResult<rhai::INT> {
        super::count_header(message, &header.to_string())
    }

    /// return the value of a header if it exists. Otherwise, returns an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// # let msg = vsmtp_mail_parser::MessageBody::try_from(concat!(
    /// "X-My-Header: 250 foo\r\n",
    /// "Subject: Unit test are cool\r\n",
    /// "\r\n",
    /// "Hello world!\r\n",
    /// # )).unwrap();
    ///
    /// # let states = vsmtp_test::vsl::run_with_msg(r#"
    /// #{
    ///   preq: [
    ///     rule "get_header" || {
    ///       accept(get_header("X-My-Header"))
    ///     }
    ///   ]
    /// }
    /// # "#, Some(msg));
    /// # use vsmtp_common::{state::State, status::Status, CodeID, Reply, ReplyCode::Code};
    /// # assert_eq!(states[&State::PreQ].2, Status::Accept(either::Right(Reply::new(
    /// #  Code { code: 250 }, "foo".to_string(),
    /// # ))));
    /// ```
    #[rhai_fn(global, name = "get_header", return_raw, pure)]
    pub fn get_header(message: &mut Message, header: &str) -> EngineResult<String> {
        Ok(vsl_guard_ok!(message.read())
            .get_header(header)
            .unwrap_or_default())
    }

    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    #[rhai_fn(global, name = "get_header", return_raw, pure)]
    pub fn get_header_obj(message: &mut Message, header: SharedObject) -> EngineResult<String> {
        get_header(message, &header.to_string())
    }

    /// Return the complete list of headers.
    #[rhai_fn(global, name = "get_all_headers", return_raw, pure)]
    pub fn get_all_headers(message: &mut Message) -> EngineResult<rhai::Array> {
        Ok(vsl_guard_ok!(message.read())
            .inner()
            .raw_headers()
            .iter()
            .map(|raw| rhai::Dynamic::from(raw.clone()))
            .collect())
    }

    /// Return a list of headers bearing the `name` given as argument.
    #[rhai_fn(global, name = "get_all_headers", return_raw, pure)]
    pub fn get_all_headers_str(message: &mut Message, name: &str) -> EngineResult<rhai::Array> {
        super::get_all_headers(message, name)
    }

    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    #[rhai_fn(global, name = "get_all_headers", return_raw, pure)]
    pub fn get_all_headers_obj(
        message: &mut Message,
        name: SharedObject,
    ) -> EngineResult<rhai::Array> {
        super::get_all_headers(message, &name.to_string())
    }

    /// add a header to the end of the raw or parsed email contained in ctx.
    #[rhai_fn(global, name = "append_header", return_raw, pure)]
    pub fn append_header(message: &mut Message, header: &str, value: &str) -> EngineResult<()> {
        super::append_header(message, &header, &value)
    }

    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    #[rhai_fn(global, name = "append_header", return_raw, pure)]
    pub fn append_header_str_obj(
        message: &mut Message,
        header: &str,
        value: SharedObject,
    ) -> EngineResult<()> {
        super::append_header(message, &header, &value.to_string())
    }

    /// prepend a header to the raw or parsed email contained in ctx.
    #[rhai_fn(global, name = "prepend_header", return_raw, pure)]
    pub fn prepend_header_str_str(
        message: &mut Message,
        header: &str,
        value: &str,
    ) -> EngineResult<()> {
        super::prepend_header(message, header, value)
    }

    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    #[rhai_fn(global, name = "prepend_header", return_raw, pure)]
    pub fn prepend_header_str_obj(
        message: &mut Message,
        header: &str,
        value: SharedObject,
    ) -> EngineResult<()> {
        super::prepend_header(message, header, &value.to_string())
    }

    /// set a header to the raw or parsed email contained in ctx.
    #[rhai_fn(global, name = "set_header", return_raw, pure)]
    pub fn set_header(message: &mut Message, header: &str, value: &str) -> EngineResult<()> {
        super::set_header(message, header, value)
    }

    #[doc(hidden)]
    #[rhai_fn(global, name = "set_header", return_raw, pure)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn set_header_str_obj(
        message: &mut Message,
        header: &str,
        value: SharedObject,
    ) -> EngineResult<()> {
        super::set_header(message, header, &value.to_string())
    }

    /// set a header to the raw or parsed email contained in ctx.
    #[rhai_fn(global, name = "rename_header", return_raw, pure)]
    pub fn rename_header(message: &mut Message, old: &str, new: &str) -> EngineResult<()> {
        super::rename_header(message, old, new)
    }

    #[doc(hidden)]
    #[rhai_fn(global, name = "rename_header", return_raw, pure)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn rename_header_str_obj(
        message: &mut Message,
        old: &str,
        new: SharedObject,
    ) -> EngineResult<()> {
        super::rename_header(message, old, &new.to_string())
    }

    #[doc(hidden)]
    #[rhai_fn(global, name = "rename_header", return_raw, pure)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn rename_header_obj_str(
        message: &mut Message,
        old: SharedObject,
        new: &str,
    ) -> EngineResult<()> {
        super::rename_header(message, &old.to_string(), new)
    }

    #[doc(hidden)]
    #[rhai_fn(global, name = "rename_header", return_raw, pure)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn rename_header_obj_obj(
        message: &mut Message,
        old: SharedObject,
        new: SharedObject,
    ) -> EngineResult<()> {
        super::rename_header(message, &old.to_string(), &new.to_string())
    }

    /// Get the message body as a string
    #[rhai_fn(global, get = "mail", return_raw, pure)]
    pub fn mail(this: &mut Message) -> EngineResult<String> {
        Ok(vsl_guard_ok!(this.read()).inner().to_string())
    }

    /// Remove a header from the raw or parsed email contained in ctx.
    #[rhai_fn(global, name = "remove_header", return_raw, pure)]
    pub fn remove_header(message: &mut Message, header: &str) -> EngineResult<bool> {
        super::remove_header(message, header)
    }

    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    #[rhai_fn(global, name = "remove_header", return_raw, pure)]
    pub fn remove_header_obj(message: &mut Message, header: SharedObject) -> EngineResult<bool> {
        super::remove_header(message, &header.to_string())
    }

    ///
    #[rhai_fn(global, return_raw, pure)]
    pub fn get_header_untouched(this: &mut Message, name: &str) -> EngineResult<rhai::Array> {
        let guard = vsl_guard_ok!(this.read());
        let name_lowercase = name.to_lowercase();

        Ok(guard
            .inner()
            .headers(true)
            .iter()
            .filter(|(key, _)| key.to_lowercase() == name_lowercase)
            .map(|(key, value)| rhai::Dynamic::from(format!("{key}:{value}")))
            .collect::<Vec<_>>())
    }
}

/// Return a list of headers bearing the `name` given as argument.
/// The `count` parameter specify the number of headers with the same name
/// to return.
fn get_all_headers(this: &mut Message, name: &str) -> EngineResult<rhai::Array> {
    let guard = vsl_guard_ok!(this.read());
    let name_lowercase = name.to_lowercase();

    Ok(guard
        .inner()
        .headers(true)
        .into_iter()
        .filter(|(key, _)| key.to_lowercase() == name_lowercase)
        .map(|(_, value)| rhai::Dynamic::from(value))
        .collect())
}

/// internal generic function to count the occurrence of a header.
fn count_header<T>(message: &mut Message, header: &T) -> EngineResult<rhai::INT>
where
    T: AsRef<str> + ?Sized,
{
    vsl_guard_ok!(message.read())
        .count_header(header.as_ref())
        .try_into()
        .map_err::<Box<rhai::EvalAltResult>, _>(|_| "header count overflowed".into())
}

/// internal generic function to append a header to the message.
fn append_header<T, U>(message: &mut Message, header: &T, value: &U) -> EngineResult<()>
where
    T: AsRef<str> + ?Sized,
    U: AsRef<str> + ?Sized,
{
    vsl_guard_ok!(message.write()).append_header(header.as_ref(), value.as_ref());
    Ok(())
}

/// internal generic function to prepend a header to the message.
fn prepend_header<T, U>(message: &mut Message, header: &T, value: &U) -> EngineResult<()>
where
    T: AsRef<str> + ?Sized,
    U: AsRef<str> + ?Sized,
{
    vsl_guard_ok!(message.write()).prepend_header(header.as_ref(), value.as_ref());
    Ok(())
}

/// internal generic function to set the value of a header.
fn set_header<T, U>(message: &mut Message, header: &T, value: &U) -> EngineResult<()>
where
    T: AsRef<str> + ?Sized,
    U: AsRef<str> + ?Sized,
{
    vsl_guard_ok!(message.write()).set_header(header.as_ref(), value.as_ref());
    Ok(())
}

/// internal generic function to rename a header.
fn rename_header<T, U>(message: &mut Message, old: &T, new: &U) -> EngineResult<()>
where
    T: AsRef<str> + ?Sized,
    U: AsRef<str> + ?Sized,
{
    vsl_guard_ok!(message.write()).rename_header(old.as_ref(), new.as_ref());
    Ok(())
}

/// internal generic function to remove a header.
fn remove_header<T>(message: &mut Message, header: &T) -> EngineResult<bool>
where
    T: AsRef<str> + ?Sized,
{
    Ok(vsl_guard_ok!(message.write()).remove_header(header.as_ref()))
}

#[cfg(test)]
mod test {
    use vsmtp_mail_parser::MessageBody;
    use vsmtp_plugin_vsl::objects::Object;

    use super::*;

    #[test]
    fn test_has_header_success() {
        let mut message = std::sync::Arc::new(std::sync::RwLock::new(MessageBody::default()));

        append_header(&mut message, "X-HEADER-1", "VALUE-1").unwrap();
        append_header_str_obj(
            &mut message,
            "X-HEADER-2",
            std::sync::Arc::new(Object::new_fqdn("example.com").unwrap()),
        )
        .unwrap();

        assert!(has_header(&mut message, "X-HEADER-1").unwrap());
        assert!(has_header(&mut message, "X-HEADER-2").unwrap());
        assert!(!has_header(&mut message, "X-HEADER-3").unwrap());
    }

    #[test]
    fn test_get_header_success() {
        let mut message = std::sync::Arc::new(std::sync::RwLock::new(MessageBody::default()));

        append_header(&mut message, "X-HEADER-1", "VALUE-1").unwrap();
        append_header_str_obj(
            &mut message,
            "X-HEADER-2",
            std::sync::Arc::new(Object::new_fqdn("example.com").unwrap()),
        )
        .unwrap();

        assert_eq!(get_header(&mut message, "X-HEADER-1").unwrap(), "VALUE-1");
        assert_eq!(
            get_header(&mut message, "X-HEADER-2").unwrap(),
            "example.com"
        );
        assert_eq!(get_header(&mut message, "X-HEADER-3").unwrap(), "");
    }

    #[test]
    fn test_append_header_success() {
        let mut message = std::sync::Arc::new(std::sync::RwLock::new(MessageBody::default()));

        append_header(&mut message, "X-HEADER-1", "VALUE-1").unwrap();
        append_header_str_obj(
            &mut message,
            "X-HEADER-2",
            std::sync::Arc::new(Object::new_fqdn("example.com").unwrap()),
        )
        .unwrap();

        assert_eq!(
            message.read().unwrap().get_header("X-HEADER-1").unwrap(),
            "VALUE-1"
        );
        assert_eq!(
            message.read().unwrap().get_header("X-HEADER-2").unwrap(),
            "example.com"
        );
    }

    #[test]
    fn test_prepend_header_success() {
        let mut message = std::sync::Arc::new(std::sync::RwLock::new(MessageBody::default()));

        prepend_header_str_str(&mut message, "X-HEADER-1", "VALUE-1").unwrap();
        prepend_header_str_obj(
            &mut message,
            "X-HEADER-2",
            std::sync::Arc::new(Object::new_fqdn("example.com").unwrap()),
        )
        .unwrap();

        assert_eq!(
            message.read().unwrap().get_header("X-HEADER-1").unwrap(),
            "VALUE-1"
        );
        assert_eq!(
            message.read().unwrap().get_header("X-HEADER-2").unwrap(),
            "example.com"
        );
    }

    #[test]
    fn test_set_header_success() {
        let mut message = std::sync::Arc::new(std::sync::RwLock::new(MessageBody::default()));

        set_header(&mut message, "X-HEADER", "VALUE-1").unwrap();
        assert_eq!(
            message.read().unwrap().get_header("X-HEADER").unwrap(),
            "VALUE-1"
        );

        set_header_str_obj(
            &mut message,
            "X-HEADER",
            std::sync::Arc::new(Object::new_fqdn("example.com").unwrap()),
        )
        .unwrap();

        assert_eq!(
            message.read().unwrap().get_header("X-HEADER").unwrap(),
            "example.com"
        );

        assert_eq!(count_header(&mut message, "X-HEADER").unwrap(), 1);
    }
}
