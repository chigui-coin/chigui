use anyhow::Result;

use chigui_core::state::State;

fn main() -> Result<()> {
    let state = State::open("./database")?;

    for tx in state.txs.iter() {
        println!("{}", tx);
    }

    let balances = state.balances.borrow();

    for (account, balance) in balances.iter() {
        println!("{}: {}", account, balance);
    }

    Ok(())
}
