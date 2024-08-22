use anyhow::{anyhow, Result};

use crate::state::AppState;

pub async fn scan_library(state: &AppState) -> Result<()> {
    // we will create a list of audiobooks
    // then we will add this list to the database
    // we need to be very careful not to reenter the same thing many times
    // which is why we will probably derive a hash from the title or something
    //
    Ok(())
}

pub async fn cleanup(state: &AppState) -> Result<()> {
    Ok(())
}
