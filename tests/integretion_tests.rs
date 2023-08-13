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

