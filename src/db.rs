// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::Path;
use std::sync::Arc;

use bpns_rocksdb::{BoundColumnFamily, Error, Store};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub device_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub room_id: String,
    pub token: String,
}

#[derive(Clone)]
pub struct DBStore {
    pub db: Store,
}

const USER_CF: &str = "user";
const SESSION_CF: &str = "session";

const COLUMN_FAMILIES: &[&str] = &[USER_CF, SESSION_CF];

impl DBStore {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            db: Store::open(path, COLUMN_FAMILIES)?,
        })
    }

    fn user_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(USER_CF)
    }

    fn session_cf(&self) -> Arc<BoundColumnFamily> {
        self.db.cf_handle(SESSION_CF)
    }

    pub fn create_session(
        &self,
        user_id: &str,
        access_token: &str,
        device_id: &str,
    ) -> Result<(), Error> {
        let value = Session {
            access_token: access_token.into(),
            device_id: device_id.into(),
        };

        self.db.put_serialized(self.session_cf(), user_id, &value)
    }

    pub fn session_exist(&self, user_id: &str) -> bool {
        self.db.get(self.session_cf(), user_id).is_ok()
    }

    pub fn get_session(&self, user_id: &str) -> Result<Session, Error> {
        self.db.get_deserialized(self.session_cf(), user_id)
    }

    /* pub fn delete_session(&self, user_id: &str) -> Result<(), Error> {
        self.db.delete(self.session_cf(), user_id)
    } */

    pub fn create_user(&self, user_id: &str, room_id: &str, token: &str) -> Result<(), Error> {
        let value: User = User {
            room_id: room_id.into(),
            token: token.into(),
        };

        self.db.put_serialized(self.user_cf(), user_id, &value)
    }

    pub fn user_exist(&self, user_id: &str) -> bool {
        self.db.get(self.user_cf(), user_id).is_ok()
    }

    pub fn user_with_room_exist(&self, user_id: &str, room_id: &str) -> bool {
        if let Ok(user) = self
            .db
            .get_deserialized::<&str, User>(self.user_cf(), user_id)
        {
            return user.room_id.as_str() == room_id;
        }

        false
    }

    pub fn delete_user(&self, user_id: &str) -> Result<(), Error> {
        self.db.delete(self.user_cf(), user_id)
    }

    pub fn get_user(&self, user_id: &str) -> Result<User, Error> {
        self.db.get_deserialized(self.user_cf(), user_id)
    }
}

impl Drop for DBStore {
    fn drop(&mut self) {
        log::trace!("Closing Database");
    }
}
