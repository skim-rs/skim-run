use std::{borrow::Cow, sync::Arc};

use skim::SkimItem;
use zbus_systemd::{zbus::Connection, zvariant::OwnedObjectPath};

use crate::SkimRun;

struct SystemdService {
    name: String,
    desc: String,
    load_state: String,
    active_state: String,
    is_user: bool,
    sub_state: String,
    follower: String,
    object_path: OwnedObjectPath,
    job_id: u32,
    job_type: String,
    job_object_path: OwnedObjectPath,
}

impl SkimItem for SystemdService {
    fn text(&self) -> Cow<str> {
        Cow::Owned(format!(
            "[{}] {} {} {}",
            if self.is_user { "user" } else { "system" },
            self.name,
            self.load_state,
            self.active_state
        ))
    }
    fn preview(&self, _context: skim::PreviewContext) -> skim::ItemPreview {
        skim::ItemPreview::Command(format!(
            "
echo Name: {}
echo Description: {}
echo States: {}, {}, {}
echo Path: {}
if [ ! -z {} ]; then echo Follower: {}; fi
if [ -ne {} 0 ]; then echo; echo Job: id {}, type {}, path {}; fi
echo
echo --- logs ---
echo
journalctl --{} -xreu {}
",
            self.name.clone(),
            self.desc.clone(),
            self.load_state.clone(),
            self.active_state.clone(),
            self.sub_state.clone(),
            self.object_path.clone(),
            self.follower.clone(),
            self.follower.clone(),
            self.job_id,
            self.job_id,
            self.job_type.clone(),
            self.job_object_path.clone(),
            if self.is_user { "user" } else { "system" },
            self.name.clone(),
        ))
    }
}

pub struct SystemdServices;

fn bind_systemctl(key: &str, cmd: &str) -> String {
    format!(
        "{}:execute(systemctl {} --{{2}} {{3}})+accept(--query {{q}} systemd-services)",
        key, cmd
    )
}

impl SkimRun for SystemdServices {
    fn set_options<'a>(
        &self,
        opts: &'a mut skim::prelude::SkimOptionsBuilder,
    ) -> &'a mut skim::prelude::SkimOptionsBuilder {
        opts.preview(Some(String::new()))
            .bind(vec![
                bind_systemctl("ctrl-r", "restart"),
                bind_systemctl("ctrl-s", "stop"),
                bind_systemctl("ctrl-s", "start"),
            ])
            .preview_window(String::from("up:80%"))
            .delimiter(String::from(r"[\[\] \t]+"))
            .header(Some(String::from(
                "systemctl - restart: ^r | stop: ^s | start: ^t",
            )))
    }
    fn get(&self) -> Vec<std::sync::Arc<dyn SkimItem>> {
        let system_units: Vec<Arc<dyn SkimItem>> = smol::block_on(async {
            let conn = Connection::system()
                .await
                .expect("Failed to connect to system bus");
            let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn)
                .await
                .unwrap();
            manager.list_units().await.to_owned()
        })
        .expect("Failed to list systemd units")
        .into_iter()
        .filter_map(
            |(
                name,
                desc,
                load_state,
                active_state,
                sub_state,
                follower,
                object_path,
                job_id,
                job_type,
                job_object_path,
            )| {
                if name.ends_with(".service") {
                    Some(Arc::new(SystemdService {
                        name,
                        desc,
                        load_state,
                        is_user: false,
                        active_state,
                        sub_state,
                        follower,
                        object_path,
                        job_id,
                        job_type,
                        job_object_path,
                    }) as Arc<dyn SkimItem>)
                } else {
                    None
                }
            },
        )
        .collect();
        let user_units: Vec<Arc<dyn SkimItem>> = smol::block_on(async {
            let conn = Connection::session()
                .await
                .expect("Failed to connect to system bus");
            let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn)
                .await
                .unwrap();
            manager.list_units().await.to_owned()
        })
        .expect("Failed to list systemd units")
        .into_iter()
        .filter_map(
            |(
                name,
                desc,
                load_state,
                active_state,
                sub_state,
                follower,
                object_path,
                job_id,
                job_type,
                job_object_path,
            )| {
                if name.ends_with(".service") {
                    Some(Arc::new(SystemdService {
                        name,
                        desc,
                        load_state,
                        is_user: true,
                        active_state,
                        sub_state,
                        follower,
                        object_path,
                        job_id,
                        job_type,
                        job_object_path,
                    }) as Arc<dyn SkimItem>)
                } else {
                    None
                }
            },
        )
        .collect();
        [system_units, user_units].into_iter().flatten().collect()
    }
}
