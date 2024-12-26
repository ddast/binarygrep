use assert_cmd;

#[test]
fn test_firstbytes() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("b8873f30")
        .arg("tests/testdata_783");
    cmd.assert().success().stdout("00000000: b8873f30\n");
    Ok(())
}

#[test]
fn test_small_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii").arg("c3df").arg("tests/testdata_783");
    cmd.assert().success().stdout("00000256: c3df\n");
    Ok(())
}

#[test]
fn test_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("c3df")
        .arg("tests/testdata_783")
        .pipe_stdin("tests/testdata_783")?;
    cmd.assert().success().stdout("00000256: c3df\n");
    Ok(())
}

#[test]
fn test_file_below_buffersize() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("7fdd")
        .arg("tests/testdata_4194304");
    cmd.assert().success()
        .stdout("0001f60d: 7fdd\n00023301: 7fdd\n0002bd5d: 7fdd\n0004c243: 7fdd\n0004cc4d: 7fdd\n0005063c: 7fdd\n000534eb: 7fdd\n0006b6d2: 7fdd\n000bcf36: 7fdd\n001122ab: 7fdd\n00119647: 7fdd\n0014a4a4: 7fdd\n00180dd1: 7fdd\n001d1d9d: 7fdd\n001d65cc: 7fdd\n001e8483: 7fdd\n001ede62: 7fdd\n00223273: 7fdd\n00228bd2: 7fdd\n002363e2: 7fdd\n00261457: 7fdd\n002716a7: 7fdd\n00271aa4: 7fdd\n002a384a: 7fdd\n002ac445: 7fdd\n002b0db8: 7fdd\n002b4902: 7fdd\n002b66f3: 7fdd\n002ba4a0: 7fdd\n002d5df5: 7fdd\n002d788b: 7fdd\n002f522c: 7fdd\n002fa2b5: 7fdd\n00334ce3: 7fdd\n00345997: 7fdd\n00347ec3: 7fdd\n0034ec4f: 7fdd\n003511a2: 7fdd\n003512cd: 7fdd\n00354773: 7fdd\n00357fbc: 7fdd\n003661ff: 7fdd\n0036a856: 7fdd\n00383561: 7fdd\n0038ea17: 7fdd\n0039d562: 7fdd\n0039f097: 7fdd\n003c224d: 7fdd\n003c5ad0: 7fdd\n003cccd9: 7fdd\n003ce7f1: 7fdd\n003f9978: 7fdd\n");
    Ok(())
}

#[test]
fn test_file_above_buffersize() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("f660")
        .arg("tests/testdata_4194310");
    cmd.assert().success()
        .stdout("00001d70: f660\n000221f4: f660\n000315d5: f660\n00048e4a: f660\n0004b1db: f660\n00057c6e: f660\n0005cdec: f660\n00064e77: f660\n0009b0b5: f660\n0009c268: f660\n000a2150: f660\n000a4bdc: f660\n000c3384: f660\n000d6b56: f660\n000e84d6: f660\n000eec55: f660\n000f1740: f660\n00116dd2: f660\n00124591: f660\n0012c019: f660\n00132cd4: f660\n0016190b: f660\n0016ce01: f660\n00178a0c: f660\n0017e5dd: f660\n001979ac: f660\n00198a5d: f660\n001a0c30: f660\n001a4752: f660\n001bb7d1: f660\n001eae63: f660\n001f7b09: f660\n00200caf: f660\n0021bd4a: f660\n0021c274: f660\n0021d443: f660\n00229842: f660\n0022d353: f660\n00258aee: f660\n0026e23d: f660\n0026f6d4: f660\n0027acfe: f660\n0028bf2f: f660\n0028cb64: f660\n00290ae6: f660\n0029750d: f660\n002aa52c: f660\n002ab705: f660\n002d89f6: f660\n002e9cd5: f660\n0030bb79: f660\n00312522: f660\n00317177: f660\n00324015: f660\n0032c292: f660\n00330602: f660\n00330d54: f660\n0033890d: f660\n00340b39: f660\n00359653: f660\n0036373d: f660\n00366d69: f660\n003690ea: f660\n003695e8: f660\n0036af3f: f660\n0038e9d7: f660\n00390343: f660\n003a0352: f660\n003bddad: f660\n003be62c: f660\n003c3563: f660\n003c6222: f660\n003c89b2: f660\n003d58b6: f660\n003fd34e: f660\n00400002: f660\n");
    Ok(())
}

#[test]
fn test_file_below_buffersize_zero() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("0000")
        .arg("tests/testdata_4194304");
    cmd.assert().success()
        .stdout("0001b694: 0000\n0003bf65: 0000\n00052251: 0000\n00054560: 0000\n000691da: 0000\n0006ff36: 0000\n00078aca: 0000\n0008bec2: 0000\n0009afc6: 0000\n000aafe1: 0000\n000c69fd: 0000\n000c8246: 0000\n000ca494: 0000\n000cdc9f: 0000\n000d4548: 0000\n000f3682: 0000\n00118fcd: 0000\n00135f0d: 0000\n00151e60: 0000\n001670b2: 0000\n0017e7a1: 0000\n00191437: 0000\n0019c389: 0000\n001b0a7e: 0000\n001b9998: 0000\n001d35ba: 0000\n001e6ec1: 0000\n001f214f: 0000\n00215163: 0000\n0021b5b0: 0000\n00247d10: 0000\n002574f8: 0000\n0026efdf: 0000\n0028ad73: 0000\n0029f4c2: 0000\n002af854: 0000\n002ca3b0: 0000\n002d9656: 0000\n002da31c: 0000\n002ea266: 0000\n002f4a42: 0000\n002f95b1: 0000\n0030f08a: 0000\n0034212f: 0000\n003460ca: 0000\n003505fd: 0000\n00352094: 0000\n003599ea: 0000\n0035c893: 0000\n0035d439: 0000\n0036493f: 0000\n00372124: 0000\n00372400: 0000\n0038c23c: 0000\n003bc7d6: 0000\n003bdee2: 0000\n003c1218: 0000\n003ca13c: 0000\n003d51dc: 0000\n003d5735: 0000\n003dbbf1: 0000\n");
    Ok(())
}

#[test]
fn test_file_above_buffersize_zero() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("0000")
        .arg("tests/testdata_4194310");
    cmd.assert().success()
        .stdout("000136b9: 0000\n0001638d: 0000\n00026c15: 0000\n0002b92c: 0000\n0004ee5c: 0000\n00052700: 0000\n00054aa7: 0000\n000753d2: 0000\n0007d9ef: 0000\n00096abf: 0000\n0009e425: 0000\n000a7c33: 0000\n000ac9ae: 0000\n000b1d61: 0000\n000b23cc: 0000\n000b60a4: 0000\n000c308d: 0000\n000e48ef: 0000\n000e9eac: 0000\n0010381c: 0000\n00111b21: 0000\n0011212b: 0000\n00112883: 0000\n00117277: 0000\n0011b72e: 0000\n0013d8fb: 0000\n00141d59: 0000\n00147f89: 0000\n0015f81c: 0000\n00162184: 0000\n0016cdb7: 0000\n0017b57b: 0000\n00182c9a: 0000\n0018fa42: 0000\n001ae27e: 0000\n001aec27: 0000\n001b3123: 0000\n001d231f: 0000\n001d4b68: 0000\n001e2686: 0000\n001e4786: 0000\n001fca6c: 0000\n00211140: 0000\n0022cdb7: 0000\n00238c8a: 0000\n0025f222: 0000\n0027c759: 0000\n002bcf09: 0000\n002bcf9b: 0000\n002d6b24: 0000\n0030b71d: 0000\n0033c696: 0000\n00355c7e: 0000\n003594ea: 0000\n00369b17: 0000\n00392e75: 0000\n003c3bef: 0000\n003c65d5: 0000\n003d1b5f: 0000\n003ec62d: 0000\n003f1448: 0000\n003f23f1: 0000\n003f4af8: 0000\n003f8b49: 0000\n003f9f9a: 0000\n");
    Ok(())
}

#[test]
fn test_large_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("c3df")
        .arg("tests/testdata_10485760");
    cmd.assert().success().stdout("0000a95d: c3df\n00022a3d: c3df\n00022cc8: c3df\n000248a9: c3df\n0002798e: c3df\n0002b0c7: c3df\n00032144: c3df\n00062150: c3df\n000621f0: c3df\n0006844c: c3df\n00079225: c3df\n0007e622: c3df\n000a2191: c3df\n000bb116: c3df\n000bed03: c3df\n000d6273: c3df\n000ddea0: c3df\n000e4b3a: c3df\n000fb575: c3df\n000fb9c5: c3df\n00114f93: c3df\n00126e17: c3df\n0012ab95: c3df\n0012b679: c3df\n0014d5cf: c3df\n0015396c: c3df\n00167845: c3df\n00172d34: c3df\n00180e1f: c3df\n001834d3: c3df\n00187c2e: c3df\n00197953: c3df\n00199712: c3df\n001aee2f: c3df\n001d4822: c3df\n001dbf1d: c3df\n001e7806: c3df\n001f50c9: c3df\n0022dd2c: c3df\n0023f17d: c3df\n00266f4c: c3df\n002a39a1: c3df\n002a9ca8: c3df\n002d7e24: c3df\n002ee36f: c3df\n002f4f4a: c3df\n003089d7: c3df\n0032779f: c3df\n00329a34: c3df\n003384f2: c3df\n00364c31: c3df\n003882dc: c3df\n003bcfd3: c3df\n003bd0f9: c3df\n003d02fb: c3df\n003d8f01: c3df\n003e9592: c3df\n00416f0a: c3df\n00417985: c3df\n0042cd94: c3df\n00458a37: c3df\n0045e747: c3df\n00465012: c3df\n0047a540: c3df\n004824e3: c3df\n00483f49: c3df\n004a4570: c3df\n004aa6c8: c3df\n004ab7b0: c3df\n004bb7f6: c3df\n004bbfa2: c3df\n004cb10d: c3df\n004da4f9: c3df\n004db103: c3df\n004f8e9f: c3df\n005066db: c3df\n005131df: c3df\n00522043: c3df\n0052e8fd: c3df\n0052f3ff: c3df\n0054356e: c3df\n00543a1f: c3df\n00553a9f: c3df\n0055c751: c3df\n00570cfb: c3df\n005878e6: c3df\n005b215f: c3df\n005f8fe1: c3df\n005fd638: c3df\n00623840: c3df\n006491c3: c3df\n0065ee47: c3df\n0067458c: c3df\n006d39ca: c3df\n006f6677: c3df\n006f9c04: c3df\n006fb947: c3df\n00708089: c3df\n00716502: c3df\n00728ad0: c3df\n0072ec4b: c3df\n00734aef: c3df\n0073b22f: c3df\n0073e482: c3df\n0073f7bb: c3df\n00741fc2: c3df\n00754b73: c3df\n0077db7c: c3df\n00787793: c3df\n0078f345: c3df\n007bbf80: c3df\n007ce714: c3df\n007ce8a8: c3df\n007ceaf6: c3df\n007d886c: c3df\n007f8729: c3df\n007ff56e: c3df\n0080b2ff: c3df\n0080e578: c3df\n0080edbc: c3df\n008182c6: c3df\n00821ac0: c3df\n008530f6: c3df\n00854bab: c3df\n0085e78f: c3df\n00886dfe: c3df\n0088718b: c3df\n00894e40: c3df\n008c1d22: c3df\n008f9500: c3df\n0092b692: c3df\n0093c9b6: c3df\n0094ffee: c3df\n00956c3b: c3df\n0097b610: c3df\n00982fc1: c3df\n00988d81: c3df\n00993f5f: c3df\n009ab51d: c3df\n009b6f15: c3df\n009d90b4: c3df\n009f541f: c3df\n");
    Ok(())
}

#[test]
fn test_buffer_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-ascii")
        .arg("9a1426792ae8bda3cbc4dc5171d62910c7ccb128e7d2f33b65bc919259284828")
        .arg("tests/testdata_10485760");
    cmd.assert()
        .success()
        .stdout("003ffff0: 9a1426792ae8bda3cbc4dc5171d62910c7ccb128e7d2f33b65bc919259284828\n");
    Ok(())
}

#[test]
fn test_ascii() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("3d9d991b").arg("tests/testdata_10485760");
    cmd.assert().success().stdout("009ff7b2: 3d9d991b  =...\n");
    Ok(())
}

#[test]
fn test_after() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("-A")
        .arg("6")
        .arg("3d9d991b")
        .arg("tests/testdata_10485760");
    cmd.assert()
        .success()
        .stdout("009ff7b2: 3d9d991bd7e15c34dfb1  =.....\\4..\n");
    Ok(())
}

#[test]
fn test_before() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("-B")
        .arg("5")
        .arg("3d9d991b")
        .arg("tests/testdata_10485760");
    cmd.assert()
        .success()
        .stdout("009ff7b2: 757307b1333d9d991b  us..3=...\n");
    Ok(())
}

#[test]
fn test_context() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("-C")
        .arg("5")
        .arg("3d9d991b")
        .arg("tests/testdata_10485760");
    cmd.assert()
        .success()
        .stdout("009ff7b2: 757307b1333d9d991bd7e15c34df  us..3=.....\\4.\n");
    Ok(())
}

#[test]
fn test_with_filename() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("-H").arg("038d2c46").arg("tests/testdata_4194310");
    cmd.assert()
        .success()
        .stdout("tests/testdata_4194310 003f9a3e: 038d2c46  ..,F\n");
    Ok(())
}

#[test]
fn test_no_offset() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--no-offset")
        .arg("038d2c46")
        .arg("tests/testdata_4194310");
    cmd.assert().success().stdout("038d2c46  ..,F\n");
    Ok(())
}

#[test]
fn test_recursive() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("binarygrep")?;
    cmd.arg("--recursive")
        .arg("deede4c1")
        .arg("tests");
    cmd.assert().success().stdout("tests/subdir/testdata_1200 000002d1: deede4c1  ....\ntests/testdata_4194304 0000d250: deede4c1  ....\n");
    Ok(())
}
