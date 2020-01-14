use bssrv::proto::ClientMessage;

fn main() {
    let string_g = "layout:0;1;west;2;3;south;4;5;north;6;7;east;8;9;north";

    let result = ClientMessage::deserialize(string_g);

    match result {
        Ok(ok) => println!("{}", ok),
        Err(err) => println!("{}", err),
    }
}
