use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Solana Raffler",
    project_url: "https://github.com/Flawm/solana_raffler/",
    contacts: "discord:VLAWMZ#1337",
    policy: "https://github.com/Flawm/solana_raffler/README.md"
}
