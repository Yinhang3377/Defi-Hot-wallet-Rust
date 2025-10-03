use defi_hot_wallet::ops::backup::*;

#[test]
fn test_backup_create() {
    let backup = Backup::new("wallet_name");
    assert_eq!(backup.wallet_name, "wallet_name"); // 瑕嗙洊 new 鏂规硶鍜屽瓧娈佃闂?}

/// 娴嬭瘯 `perform_backup` 鍗犱綅鍑芥暟銆?/// 杩欎釜娴嬭瘯楠岃瘉浜嗗崰浣嶅嚱鏁板綋鍓嶆€绘槸杩斿洖鎴愬姛 (`Ok(())`)锛?/// 纭繚浜嗗嵆浣垮湪妯℃嫙瀹炵幇涓嬶紝鍏惰涓轰篃鏄彲棰勬祴鐨勩€?#[test]
fn test_perform_backup_function() {
    let backup = Backup::new("any_wallet_name");
    assert_eq!(perform_backup(&backup), Ok(())); // 瑕嗙洊 perform_backup 鍑芥暟
}
