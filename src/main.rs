use bssrv::proto::{ClientMessage, ServerMessage};
use bssrv::types::{Position, RestoreState, Who, Hits};

fn main() {
    let string_g = "layout:5;AC;0;1;west;PB;2;3;south;C;4;5;north;D;6;7;east;B;8;9;north";

    let result = ClientMessage::deserialize(string_g);

    match result {
        Ok(ok) => {
            if let ClientMessage::Layout(layout) = ok.clone() {
                let l = layout.clone();

                println!("{}", ServerMessage::RestoreSessionOk(RestoreState::Game {
                    on_turn: Who::You,
                    player_board: Hits::new(Vec::new()),
                    layout,
                    opponent_board: Hits::new(Vec::new()),
                    sunk_ships: l.placements().clone()
                }).serialize())
            }
            println!("{}", ok);
        },
        Err(err) => println!("{}", err),
    }


    let message = ServerMessage::OpponentHit(Position::new(2, 4).unwrap());

    println!("{}", message.serialize());
}
