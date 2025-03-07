// Copyright 2023 Zinc Labs Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::cli::data::cli::Cli;

pub mod cli;
pub mod export;
pub mod import;

pub trait Context {
    fn operator(c: Cli) -> Result<bool, anyhow::Error>;
}

#[cfg(test)]
mod tests {
    use crate::cli::data::cli::Cli;
    use crate::cli::data::Context;
    use crate::cli::data::export::Export;
    use crate::cli::data::import::Import;

    fn test_export_operator() {
        let args = vec![
            "openobserve",
            "--c", "stream",
            "--o", "default",
            "--st", "logs",
            "--s", "default",
            "--t", "json",
            "--f", "day",
            "--d", "./json",
        ];

        let cli = Cli::args(args);

        if let Err(err) = Export::operator(cli.clone()) {
            println!("Error: {}", err);
        }
    }

    fn test_import_operator() {
        let args = vec![
            "openobserve",
            "--c", "stream",
            "--o", "default",
            "--st", "logs",
            "--s", "default",
            "--t", "json",
            "--f", "day",
            "--d", "./json",
        ];

        let cli = Cli::args(args);

        if let Err(err) = Import::operator(cli.clone()) {
            println!("Error: {}", err);
        }
    }
}