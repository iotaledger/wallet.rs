/// create a line error with the file and the line number
#[macro_export]
macro_rules! line_error {
    () => {
        concat!("Error at ", file!(), ":", line!())
    };
    ($str:expr) => {
        concat!($str, " @", file!(), ":", line!())
    };
}

mod client;
mod connection;
mod provider;
mod snap;
mod state;

use client::Client;
use provider::Provider;
use snap::{deserialize_from_snapshot, get_snapshot_path, serialize_to_snapshot};

use vault::{Base64Decodable, Id, Key};

// handle the encryption command.
pub fn encrypt(pass: &str, plain: impl AsRef<str>) -> Id {
    let snapshot = get_snapshot_path();

    if snapshot.exists() {
        let snapshot = get_snapshot_path();
        let client: Client<Provider> = deserialize_from_snapshot(&snapshot, pass);

        let id = client.create_record(plain.as_ref().as_bytes().to_vec());

        let snapshot = get_snapshot_path();
        serialize_to_snapshot(&snapshot, pass, client);
        id
    } else {
        let key = Key::<Provider>::random().expect("Unable to generate a new key");
        let id = Id::random::<Provider>().expect("Unable to generate a new id");
        let client = Client::create_chain(key, id);
        let id = client.create_record(plain.as_ref().as_bytes().to_vec());

        let snapshot = get_snapshot_path();
        serialize_to_snapshot(&snapshot, pass, client);
        id
    }
}

// handle the read command.
pub fn read(pass: &str, id: impl AsRef<str>) -> Option<String> {
    let snapshot = get_snapshot_path();
    let client: Client<Provider> = deserialize_from_snapshot(&snapshot, pass);

    let id =
        Vec::from_base64(id.as_ref().as_bytes()).expect("couldn't convert the id to from base64");
    let id = Id::load(&id).expect("Couldn't build a new Id");

    let record = client.read_record_by_id(id);

    let snapshot = get_snapshot_path();
    serialize_to_snapshot(&snapshot, pass, client);
    record
}

// create a record with a revoke transaction.  Data isn't actually deleted until it is garbage collected.
pub fn revoke(pass: &str, id: impl AsRef<str>) {
    let snapshot = get_snapshot_path();
    let client: Client<Provider> = deserialize_from_snapshot(&snapshot, pass);

    let id =
        Vec::from_base64(id.as_ref().as_bytes()).expect("couldn't convert the id to from base64");
    let id = Id::load(&id).expect("Couldn't build a new Id");

    client.revoke_record_by_id(id);

    let snapshot = get_snapshot_path();
    serialize_to_snapshot(&snapshot, pass, client);
}
