use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rd_driver::{
    cmd::{self, Ctx},
    wallet::{Me, Parties},
};
use serde_json::Value;

/// RD/DFABI confidential wallet, BI audit and negative-test driver for Iroha 2.
/// Every command prints one line `result_json=<json>` on success.
#[derive(Parser)]
#[command(name = "rd2")]
struct Cli {
    /// Torii base URL (e.g. http://127.0.0.1:8080)
    #[arg(long, global = true, default_value = "http://127.0.0.1:8080")]
    torii: String,
    /// Chain id (network/public/chain_id)
    #[arg(long, global = true, default_value = "")]
    chain: String,
    /// network/public/parties.json
    #[arg(long, global = true, default_value = "parties.json")]
    parties: PathBuf,
    /// Party wallet dir (account.json, view.json, spend.key, notes.json; BI: audit-v1.json)
    #[arg(long, global = true, default_value = ".")]
    wallet: PathBuf,
    /// This party's name in parties.json (bi, bank-a, ...)
    #[arg(long, global = true, default_value = "")]
    party: String,
    #[command(subcommand)]
    cmd: Sub,
}

#[derive(Subcommand)]
enum Sub {
    /// Create a party wallet dir with fresh keys (tests). Prints the public halves for parties.json.
    Keygen {
        #[arg(long)]
        audit_key: bool,
    },
    /// Print this wallet's public receive address (merge into parties.json as "address").
    Address,
    /// BI: Mint to BI, then Shield to a party's address, envelope in tx metadata.
    Issue {
        #[arg(long)]
        asset: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: u128,
        #[arg(long, default_value = "issue")]
        reference: String,
        #[arg(long)]
        to_owner_tag: Option<String>,
        #[arg(long)]
        to_diversifier: Option<String>,
    },
    /// Rebuild the leaf list from blocks, check the root, trial-decrypt envelopes, store notes.json.
    Scan {
        #[arg(long)]
        asset: String,
    },
    /// Private transfer (1-2 input notes, recipient + change output).
    Transfer {
        #[arg(long)]
        asset: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: u128,
        #[arg(long, default_value = "transfer")]
        reference: String,
        #[arg(long)]
        to_owner_tag: Option<String>,
        #[arg(long)]
        to_diversifier: Option<String>,
    },
    /// Exit whole notes to a public balance (amount must equal the sum of 1-2 notes).
    Unshield {
        #[arg(long)]
        asset: String,
        #[arg(long)]
        amount: u128,
        /// Party to credit (default: self)
        #[arg(long)]
        to: Option<String>,
        #[arg(long, default_value = "unshield")]
        reference: String,
    },
    /// BI: decrypt every envelope, recompute commitments, check conservation, print violations.
    Audit {
        /// Asset names or ids (repeatable)
        #[arg(long, required = true)]
        asset: Vec<String>,
        /// Dir with audit-v<N>.json (default: --wallet)
        #[arg(long)]
        audit_dir: Option<PathBuf>,
    },
    /// Submit the attacks the node can reject; print expected vs got.
    Negative {
        #[arg(long)]
        asset: String,
        /// A second asset for the wrong-asset-tag case
        #[arg(long)]
        other_asset: String,
        #[arg(long)]
        to: String,
        #[arg(long, default_value_t = 1)]
        amount: u128,
        #[arg(long)]
        to_owner_tag: Option<String>,
        #[arg(long)]
        to_diversifier: Option<String>,
    },
}

fn run(cli: Cli) -> eyre::Result<Value> {
    let party = cli.party.clone();
    if let Sub::Keygen { audit_key } = &cli.cmd {
        return cmd::keygen(&cli.wallet, &party, *audit_key);
    }
    let me = Me::load(&cli.wallet, &party)?;
    if let Sub::Address = &cli.cmd {
        return Ok(cmd::address(&me));
    }
    let c = Ctx { torii: cli.torii, chain: cli.chain, parties: Parties::load(&cli.parties)?, me };
    if c.chain.is_empty() {
        return Err(eyre::eyre!("--chain is required"));
    }
    match cli.cmd {
        Sub::Issue { asset, to, amount, reference, to_owner_tag, to_diversifier } => {
            cmd::issue(&c, &asset, &to, amount, &reference, to_owner_tag.as_deref(), to_diversifier.as_deref())
        }
        Sub::Scan { asset } => cmd::scan(&c, &asset),
        Sub::Transfer { asset, to, amount, reference, to_owner_tag, to_diversifier } => {
            cmd::transfer(&c, &asset, &to, amount, &reference, to_owner_tag.as_deref(), to_diversifier.as_deref())
        }
        Sub::Unshield { asset, amount, to, reference } => cmd::unshield(&c, &asset, amount, to.as_deref(), &reference),
        Sub::Audit { asset, audit_dir } => {
            let dir = audit_dir.unwrap_or_else(|| cli.wallet.clone());
            cmd::audit_cmd(&c, &asset, &dir)
        }
        Sub::Negative { asset, other_asset, to, amount, to_owner_tag, to_diversifier } => {
            cmd::negative(&c, &asset, &other_asset, &to, amount, to_owner_tag.as_deref(), to_diversifier.as_deref())
        }
        Sub::Keygen { .. } | Sub::Address => unreachable!(),
    }
}

fn main() {
    match run(Cli::parse()) {
        Ok(v) => println!("result_json={v}"),
        Err(e) => {
            println!("result_json={}", serde_json::json!({"error": format!("{e:#}")}));
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}
