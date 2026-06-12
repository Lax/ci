use anyhow::Result;

use crate::db::store::FreqDb;

/// Run `rime_dict_manager -s` to trigger Rime sync,
/// then read the exported .userdb.txt files.
pub fn import(_freq_db: &mut FreqDb, _rime_user_dir: &str) -> Result<()> {
    // TODO:
    // 1. std::process::Command("rime_dict_manager").arg("-s").output()
    // 2. Walk sync/{installation_id}/*.userdb.txt
    // 3. Parse each file (TSV format: code\tword\tc=N d=N t=N)
    // 4. Convert to Entry + merge into freq_db
    // 5. Update device info
    Ok(())
}

/// Generate .userdb.txt from freq_db entries,
/// then run `rime_dict_manager -r` to merge into Rime.
pub fn export(_freq_db: &FreqDb, _output_path: &str) -> Result<()> {
    // TODO:
    // 1. Generate Rime-format .userdb.txt (metadata + entries)
    // 2. std::process::Command("rime_dict_manager").arg("-r").arg(output_path).output()
    // 3. std::process::Command("rime_dict_manager").arg("-s").output()
    Ok(())
}
