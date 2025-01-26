use ethers::types::Address;

pub fn get_address_strings(addr: &Address) -> (String, String) {
    (format!("{:#x}", addr), addr.to_string())
}