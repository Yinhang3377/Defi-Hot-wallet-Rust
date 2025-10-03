use defi_hot_wallet::security::anti_debug::is_debugger_present;

#[test]
fn test_is_debugger_present() {
    // 杩欎釜娴嬭瘯鍙槸纭鍑芥暟鑳藉杩愯锛屼笉鍒ゆ柇缁撴灉鐨勬纭€?    let result = is_debugger_present();
    // 鏃犺缁撴灉鏄粈涔堬紝鍙鍑芥暟涓嶅穿婧冨氨绠楅€氳繃
    println!("Debugger present: {}", result);
}
