// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use protocol::sessionsrv;
use sessionsrv::data_store::DataStore;

#[test]
fn create_account() {
    let ds = datastore_test!(DataStore);
    let mut ac = sessionsrv::AccountCreate::new();
    ac.set_email(String::from("bobo@chef.io"));
    ac.set_name(String::from("Bobo T. Clown"));

    let account = ds.create_account(&ac).expect("Should create account");

    assert_eq!(account.get_email(), "bobo@chef.io");
    assert_eq!(account.get_name(), "Bobo T. Clown");
}

fn create_bobo_account(ds: &DataStore) -> sessionsrv::Account {
    let mut ac = sessionsrv::AccountCreate::new();
    ac.set_email(String::from("bobo@chef.io"));
    ac.set_name(String::from("Bobo T. Clown"));
    ds.create_account(&ac).expect("Should create account")
}

#[test]
fn get_account() {
    let ds = datastore_test!(DataStore);
    let bobo = create_bobo_account(&ds);

    let mut ag = sessionsrv::AccountGet::new();
    ag.set_name(bobo.get_name().to_string());
    let bobo2 = ds
        .get_account(&ag)
        .expect("Should run without error")
        .expect("Bobo should exist");

    assert_eq!(bobo.get_id(), bobo2.get_id());
    assert_eq!(bobo.get_email(), bobo2.get_email());
    assert_eq!(bobo.get_name(), bobo2.get_name());
}

#[test]
fn get_account_by_id() {
    let ds = datastore_test!(DataStore);
    let bobo = create_bobo_account(&ds);

    let mut ag = sessionsrv::AccountGetId::new();
    ag.set_id(bobo.get_id());
    let bobo2 = ds
        .get_account_by_id(&ag)
        .expect("Should run without error")
        .expect("Bobo should exist");

    assert_eq!(bobo.get_id(), bobo2.get_id());
    assert_eq!(bobo.get_email(), bobo2.get_email());
    assert_eq!(bobo.get_name(), bobo2.get_name());
}
