use defi_hot_wallet::audit::rollback::*;

#[test]
fn test_rollback_new() {
    let rollback = Rollback::new("tx_id");
    assert_eq!(rollback.tx_id, "tx_id"); // 瑕嗙洊 new 鏂规硶鍜屽瓧娈佃闂?}

/// 娴嬭瘯 `rollback_tx` 鍗犱綅鍑芥暟銆?/// 杩欎釜娴嬭瘯楠岃瘉浜嗗崰浣嶅嚱鏁板綋鍓嶆€绘槸杩斿洖鎴愬姛 (`Ok(())`)锛?/// 纭繚浜嗗嵆浣垮湪妯℃嫙瀹炵幇涓嬶紝鍏惰涓轰篃鏄彲棰勬祴鐨勩€?#[test]
fn test_rollback_tx_function() {
    assert_eq!(rollback_tx("any_tx_id"), Ok(()));
}
