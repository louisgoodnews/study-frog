//! Performing the profile effects. The worker's `config_dir` is the folder of the profile in use;
//! `root_config` is the folder above all profiles.

use std::fs;

use studyfrog_config::profiles::Profiles;
use studyfrog_core::settings::AppSettings;
use studyfrog_store::app_config::{
    profile_dir, read_app_settings, read_profiles, write_app_settings, write_profiles,
};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::profile_ui::ProfileMsg;

/// The profiles, then the settings of the one in use: what the application needs at start-up.
pub fn startup(w: &mut Worker) -> Vec<Msg> {
    let Some(root) = w.root_config.clone() else {
        return vec![Msg::Started {
            app: AppSettings::default(),
            error: None,
        }];
    };
    let (profiles, error) = match read_profiles(&root) {
        Ok(p) => (p, None),
        Err(e) => (Profiles::default(), Some(e.to_string())),
    };
    w.config_dir = Some(profile_dir(&root, &profiles.current));
    let dir = w.config_dir.clone().unwrap_or_default();
    let (app, app_error) = match read_app_settings(&dir) {
        Ok(app) => (app, None),
        Err(e) => (AppSettings::default(), Some(e.to_string())),
    };
    vec![
        Msg::Profile(ProfileMsg::Loaded(profiles)),
        Msg::Started {
            app,
            error: error.or(app_error),
        },
    ]
}

fn changed(result: Result<(Profiles, AppSettings), String>) -> Msg {
    Msg::Profile(ProfileMsg::Changed(result))
}

/// Use `name`: remember it as the current profile and read its settings.
pub fn switch(w: &mut Worker, name: &str) -> Msg {
    let Some(root) = w.root_config.clone() else {
        return changed(Err("there is no config folder".to_owned()));
    };
    let result = read_profiles(&root)
        .map_err(|e| e.to_string())
        .and_then(|p| {
            if !p.names.iter().any(|n| n == name) {
                return Err(format!("there is no profile called \"{name}\""));
            }
            let next = Profiles {
                current: name.to_owned(),
                ..p
            };
            let dir = profile_dir(&root, name);
            let app = read_app_settings(&dir).map_err(|e| e.to_string())?;
            write_profiles(&root, &next).map_err(|e| e.to_string())?;
            w.config_dir = Some(dir);
            Ok((next, app))
        });
    changed(result)
}

/// Add a profile with default settings and switch to it.
pub fn create(w: &mut Worker, name: &str) -> Msg {
    let Some(root) = w.root_config.clone() else {
        return changed(Err("there is no config folder".to_owned()));
    };
    let result = read_profiles(&root)
        .map_err(|e| e.to_string())
        .and_then(|mut p| {
            if let Some(why) = studyfrog_config::profiles::name_problem(name, &p.names) {
                return Err(why);
            }
            p.names.push(name.trim().to_owned());
            let dir = profile_dir(&root, name.trim());
            write_app_settings(&dir, &AppSettings::default()).map_err(|e| e.to_string())?;
            write_profiles(
                &root,
                &Profiles {
                    current: name.trim().to_owned(),
                    ..p.clone()
                },
            )
            .map_err(|e| e.to_string())?;
            w.config_dir = Some(dir);
            Ok((
                Profiles {
                    current: name.trim().to_owned(),
                    ..p
                },
                AppSettings::default(),
            ))
        });
    changed(result)
}

/// Remove a profile (never the default one or the one in use) with its settings and keys.
pub fn delete(w: &mut Worker, name: &str) -> Msg {
    let Some(root) = w.root_config.clone() else {
        return changed(Err("there is no config folder".to_owned()));
    };
    let result = read_profiles(&root)
        .map_err(|e| e.to_string())
        .and_then(|mut p| {
            if name == studyfrog_config::profiles::DEFAULT_PROFILE || name == p.current {
                return Err("the default profile and the one in use cannot be deleted".to_owned());
            }
            p.names.retain(|n| n != name);
            let dir = profile_dir(&root, name);
            if dir != root && dir.starts_with(root.join("profiles")) {
                let _ = fs::remove_dir_all(&dir);
            }
            write_profiles(&root, &p).map_err(|e| e.to_string())?;
            let app = read_app_settings(w.config_dir.as_deref().unwrap_or(&root))
                .map_err(|e| e.to_string())?;
            Ok((p, app))
        });
    changed(result)
}
