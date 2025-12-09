
// use anyhow::{bail, Result};
// use hex::{decode as hex_decode, encode as hex_encode};
// use sha2::{Digest, Sha256};
// use ripemd::Ripemd160;
// use std::collections::HashMap;
// use std::io::{self, Write};
// // use std::hash::Hash;
// use secp256k1::Message;
// use secp256k1::Secp256k1;

// use bitcoin::hashes::{sha256d, Hash};

// type Stack = Vec<Vec<u8>>;

// macro_rules! lazy_static {
//     ($init:expr) => {
//         std::sync::OnceLock::from($init)
//     };
// }


// static OPCODE_MAP: std::sync::OnceLock<HashMap<u8, &'static str>> = std::sync::OnceLock::new();
// static REVERSE_OPCODE_MAP: std::sync::OnceLock<HashMap<&'static str, u8>> = std::sync::OnceLock::new();

// fn init_opcodes() {
//     let mut op = HashMap::new();
//     op.insert(0x00, "OP_0");
//     for i in 1..=16 {
//         op.insert(0x50 + i, match i {
//             1 => "OP_1",
//             2 => "OP_2",
//             3 => "OP_3",
//             4 => "OP_4",
//             5 => "OP_5",
//             6 => "OP_6",
//             7 => "OP_7",
//             8 => "OP_8",
//             9 => "OP_9",
//             10 => "OP_10",
//             11 => "OP_11",
//             12 => "OP_12",
//             13 => "OP_13",
//             14 => "OP_14",
//             15 => "OP_15",
//             16 => "OP_16",
//             _ => unreachable!(),
//         });
//     }
//     op.insert(0x76, "OP_DUP");
//     op.insert(0x87, "OP_EQUAL");
//     op.insert(0x88, "OP_EQUALVERIFY");
//     op.insert(0xac, "OP_CHECKSIG");
//     op.insert(0xae, "OP_CHECKMULTISIG");
//     op.insert(0xa9, "OP_HASH160");
//     op.insert(0x6a, "OP_RETURN");

//     let mut rev = HashMap::new();
//     for (&byte, &name) in &op {
//         rev.insert(name, byte);
//     }
//     rev.insert("OP_FALSE", 0x00);
//     rev.insert("OP_TRUE", 0x51);

//     let _ = OPCODE_MAP.set(op);
//     let _ = REVERSE_OPCODE_MAP.set(rev);
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum ScriptType {
//     P2PK,
//     P2PKH,
    
//     P2SH,
//     P2MS,
//     Return,
//     Unknown,
// }

// #[derive(Clone)]
// struct Script {
//     hex: String,
//     asm: Vec<String>,
//     script_type: ScriptType,
// }

// fn hash160(data: &[u8]) -> Vec<u8> {
//     let sha = Sha256::digest(data);
//     let mut ripemd = Ripemd160::new();
//     ripemd.update(sha);  //feed the SHA-256 hash into RIPEMD-160.
//     ripemd.finalize().to_vec() //compute the final RIPEMD-160 hash and return it as a Vec<u8> (vector of bytes).
// }

// impl Script {
//     fn new(input: &str) -> Result<Self> {
//         let trimmed = input.trim();
//         if trimmed.contains(' ') || trimmed.contains("OP_") {
//             Self::from_asm(trimmed)
//         } else {
//             Self::from_hex(trimmed)
//         }
//     }

//     fn from_hex(hex_str: &str) -> Result<Self> {
//         let bytes = hex_decode(hex_str)?;
//         let asm = Self::bytes_to_asm(&bytes);
//         let script_type = Self::detect_type(&asm);
//         Ok(Script {
//             hex: hex_str.to_ascii_lowercase(),
//             asm,
//             script_type,
//         })
//     }

//     fn from_asm(asm_str: &str) -> Result<Self> {
//         let asm: Vec<String> = asm_str.split_whitespace().map(|s| s.to_string()).collect();
//         let bytes = Self::asm_to_bytes(&asm)?;
//         let hex = hex_encode(&bytes);
//         let script_type = Self::detect_type(&asm);
//         Ok(Script { hex, asm, script_type })
//     }

//     fn bytes_to_asm(bytes: &[u8]) -> Vec<String> {
//         let mut asm = Vec::new();
//         let mut i = 0;
//         while i < bytes.len() {
//             let op = bytes[i];
//             i += 1;

//             if op >= 0x01 && op <= 0x4b {
//                 let len = op as usize;
//                 if i + len > bytes.len() { 
//                     break; 
//                 }
//                 let data = &bytes[i..(i + len)];
//                 asm.push(hex_encode(data));
//                 i += len;
//             } else if let Some(&name) = OPCODE_MAP.get().and_then(|m| m.get(&op)) {
//                 asm.push(name.to_string());
//             } else {
//                 asm.push(format!("{:02x}", op));
//             }
//         }
//         asm
//     }

//     fn asm_to_bytes(asm: &[String]) -> Result<Vec<u8>> {
//         let mut bytes = Vec::new();
//         for part in asm {
//             if let Some(&code) = REVERSE_OPCODE_MAP.get().and_then(|m| m.get(part.as_str())) {
//                 bytes.push(code);
//             } else if let Ok(n) = part.strip_prefix("OP_").unwrap_or("").parse::<u8>() {
//                 if n <= 16 {
//                     bytes.push(0x50 + n);
//                 } else {
//                     bail!("Invalid OP_n");
//                 }
//             } else {
//                 // Raw data push
//                 let data = hex_decode(part)?;
//                 let len = data.len();
//                 if len < 0x4c {
//                     bytes.push(len as u8);
//                 } else if len <= 0xff {
//                     bytes.push(0x4c);
//                     bytes.push(len as u8);
//                 } else {
//                     bail!("Data too large");
//                 }
//                 bytes.extend_from_slice(&data);
//             }
//         }
//         Ok(bytes)
//     }

//     fn detect_type(asm: &[String]) -> ScriptType {
//         let op_ch="OP_CHECKSIG".to_string();
//         let op_dup="OP_DUP".to_string();
//         let op_has="OP_HASH160".to_string();
//         let op_eq="OP_EQUALVERIFY".to_string();
//         let op_equ="OP_EQUAL".to_string();
//         let op_ren="OP_RETURN".to_string();

//        match asm {
//             [_, op_ch] => ScriptType::P2PK,

//             [op_dup, op_has, _, op_eq, op_ch] => ScriptType::P2PKH,

//             [op_has, _, op_equ] => ScriptType::P2SH,

//             _ if asm.last().map_or(false, |s| s == "OP_CHECKMULTISIG") => ScriptType::P2MS,

//             [op_ren, ..] => ScriptType::Return,

//             _ => ScriptType::Unknown,
//         }
//     }

//     fn run(scripts: &[Script], debug: bool) -> Result<Stack> {
//         let mut full_script: Vec<String> = scripts.iter().flat_map(|s| s.asm.clone()).collect();
//         let mut stack: Stack = Vec::new();

//         while let Some(op) = full_script.first().cloned() {
//             full_script.remove(0);

//             let executed = if let Some(&code) = REVERSE_OPCODE_MAP.get().and_then(|m| m.get(op.as_str())) {
//                 match code {
//                     0x76 => { // OP_DUP
//                         let top = stack.last().ok_or_else(|| anyhow::anyhow!("OP_DUP on empty stack"))?.clone();
//                         stack.push(top);
//                         true
//                     }
//                     0xa9 => { // OP_HASH160
//                         let elem = stack.pop().ok_or_else(|| anyhow::anyhow!("OP_HASH160 on empty stack"))?;
//                         stack.push(hash160(&elem));
//                         true
//                     }
//                     0x87 => { // OP_EQUAL
//                         let b = stack.pop().unwrap();
//                         let a = stack.pop().unwrap();
//                         stack.push(if a == b { vec![1u8] } else { vec![] });
//                         true
//                     }
//                     0x88 => { // OP_EQUALVERIFY
//                         let b = stack.pop().unwrap();
//                         let a = stack.pop().unwrap();
//                         if a != b {
//                             bail!("OP_EQUALVERIFY failed");
//                         }
//                         true
//                     }
//                     0xac => { // OP_CHECKSIG – fake success
//                         // stack.pop();
//                         // stack.pop();
//                         // stack.push(vec![1u8]);
//                         // true


//                         // let sig = stack.pop().ok_or_else(|| anyhow::anyhow!("Empty stack"))?;
//                         // let pubkey = stack.pop().ok_or_else(|| anyhow::anyhow!("Empty stack"))?;

//                         // // stack.push(vec![1]);
//                         // // stack.push(if pubkey == sig { vec![1u8] } else { vec![] });

//                         // // Strip sighash byte if present
//                         // // let sig = if !sig.is_empty() && sig[0] & 0x80 != 0 || sig.len() > 65 {
//                         // //     &sig[..sig.len()-1]
//                         // // } else {
//                         // //     &sig
//                         // // };
//                         // let sig_no_sighash = &sig[..sig.len()-1];

//                         // // Deterministic message: hash of "Bitcoin" + pubkey + sig (common trick in demos)
//                         // let mut hash_input = b"Bitcoin".to_vec();
//                         // hash_input.extend_from_slice(&pubkey);
//                         // hash_input.extend_from_slice(sig_no_sighash);
//                         // let msg_hash = sha256d::Hash::hash(&hash_input);
//                         // let msg = Message::from_slice(&msg_hash[..])?;

//                         // let secp = Secp256k1::new();

//                         // let Ok(pk) = secp256k1::PublicKey::from_slice(&pubkey) else {
//                         //     stack.push(vec![]);
//                         //     continue;
//                         // };
//                         // let Ok(sig) = secp256k1::ecdsa::Signature::from_der(sig_no_sighash) else {
//                         //     stack.push(vec![]);
//                         //     continue;
//                         // };

//                         // let verified = secp.verify_ecdsa(&msg, &sig, &pk).is_ok();
//                         // stack.push(if verified { vec![1u8] } else { vec![] });
//                         // true


//                         let sig = stack.pop().ok_or_else(|| anyhow::anyhow!("Empty stack"))?;
//                         let pubkey = stack.pop().ok_or_else(|| anyhow::anyhow!("Empty stack"))?;

//                         let sig_no_sighash = &sig[..sig.len()-1]; // strip sighash byte

//                         // For demo purposes, hash "Bitcoin" + pubkey + sig
//                         let mut hash_input = b"Bitcoin".to_vec();
//                         hash_input.extend_from_slice(&pubkey);
//                         hash_input.extend_from_slice(sig_no_sighash);
//                         let msg_hash = sha256d::Hash::hash(&hash_input);
//                         let msg = Message::from_slice(&msg_hash[..])?;

//                         let secp = Secp256k1::new();

//                         let verified = if let Ok(pk) = secp256k1::PublicKey::from_slice(&pubkey) {
//                             if let Ok(sig) = secp256k1::ecdsa::Signature::from_der(sig_no_sighash) {
//                                 secp.verify_ecdsa(&msg, &sig, &pk).is_ok()
//                             } else { false }
//                         } else { false };

//                         stack.push(if verified { vec![1u8] } else { vec![] });
//                         true
//                     }
                    
//                     // 0xae => { // OP_CHECKMULTISIG – fake + off-by-one bug
//                     //     let n = stack.pop().unwrap()[0] as usize - 0x50;

//                     //     for _ in 0..n { stack.pop(); }

//                     //     let m = stack.pop().unwrap()[0] as usize - 0x50;

//                     //     for _ in 0..m { stack.pop(); }

//                     //     stack.pop(); // extra pop – Bitcoin bug emulation
//                     //     stack.push(vec![1u8]);
//                     //     true
//                     // }
//                     0x6a => bail!("OP_RETURN makes script invalid"),
//                     _ => false,
//                 }
//             } else {
//                 false
//             };

//             if !executed {
//                 // Must be pushed data
//                 let data = hex_decode(&op)?;
//                 stack.push(data);
//             }

//             if debug {
//                 Self::debug_print(&full_script, &stack);
//                 let mut dummy = String::new();
//                 io::stdin().read_line(&mut dummy).ok();
//             }
//         }

//         Ok(stack)
//     }

//     fn debug_print(remaining: &[String], stack: &Stack) {
//         // print!("\x1B[2J\x1B[H"); // clear screen
//         println!("Remaining script: {remaining:?}\n");
//         println!("Stack (top → bottom):");
//         if stack.is_empty() {
//             println!("  <empty>");
//         } else {
//             for item in stack.iter().rev() {
//                 println!("  {}", hex_encode(item));
//             }
//         }
//         println!("\nPress Enter for next step...");
//         io::stdout().flush().unwrap();
//     }

//     fn validate(stack: &Stack) -> bool {
//         !stack.is_empty() && !stack.last().unwrap().is_empty()
//     }
// }

// fn main() -> Result<()> {
//     init_opcodes();

//     println!("Bitcoin Script Interpreter (Rust)\n");

//     print!("Locking script (hex or asm): ");
//     io::stdout().flush()?;
//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;
//     let locking = Script::new(&input)?;

//     println!("Type: {:?}\n", locking.script_type);

//     print!("Unlocking script (hex or asm): ");
//     io::stdout().flush()?;
//     input.clear();
//     io::stdin().read_line(&mut input)?;
//     let unlocking = Script::new(&input)?;

//     println!("\n=== Scripts ===");
//     println!("Locking : {}", locking.asm.join(" "));
//     println!("Unlocking: {}", unlocking.asm.join(" "));
//     println!("\nPress Enter to start execution...");
//     io::stdin().read_line(&mut String::new())?;

//     let final_stack = if locking.script_type == ScriptType::P2SH {
//         // Very simple P2SH handling – assumes redeem script is last push in unlocking scriptSig
//         let redeem_hex = unlocking.asm.last().unwrap();
//         let redeem_script = Script::from_hex(redeem_hex)?;
//         Script::run(&[unlocking.clone(), redeem_script], true)?
//     } else {
//         Script::run(&[unlocking.clone(), locking.clone()], true)?
//     };

//     println!("\n=== Final stack ===");
//     for item in final_stack.iter().rev() {
//         println!("  {}", hex_encode(item));
//     }

//     if Script::validate(&final_stack) {
//         println!("\nVALID – Transaction would be accepted");
//     } else {
//         println!("\nINVALID – Transaction rejected");
//     }

    
//     Ok(())
// }


// // # EXAMPLE SCRIPTS (Standard)
// // # ---------------

// // # p2pk: 4104240ac91558e66c0628693cee5f5120d43caf73cad8586f9f56a447cc6b926520d2b3b259874e5d79dfb4b9aff3405a10cbce47ee820e0824dc7004d5bbcea86fac
// // # p2pk (unlock): 4730440220277c967dda11986e06e508235006b7e83bc27a1cb0ffaa0d97a543e178199b6a022040d4f8f17865e45de9ca7bcfe3ee2228e175cfcb4468b7650f09b534d3f71f4401

// // # p2pkh: 76a91491ef7f43180d71d61ca3870a1b0445c116efa78088ac
// // # p2pkh (unlock):

// // # p2sh: a914e9c3dd0c07aac76179ebc76a6c78d4d67c6c160a87
// // # p2sh (unlock): 00483045022100ad0851c69dd756b45190b5a8e97cb4ac3c2b0fa2f2aae23aed6ca97ab33bf88302200b248593abc1259512793e7dea61036c601775ebb23640a0120b0dba2c34b79001455141042f90074d7a5bf30c72cf3a8dfd1381bdbd30407010e878f3a11269d5f74a58788505cdca22ea6eab7cfb40dc0e07aba200424ab0d79122a653ad0c7ec9896bdf51ae

// // # p2ms: 5141204e00003bf2a106de6a91d6b7d3d8f067e70fd40ab0bd7c12f278c35eba8e16e1cd73e5d9871f1f2a027659bce210737856849248260a58e973a9a37a6fbca6354100d8fbd53efe72e1fd664c935e929b2c41b050f5813c93b2d3e8128b3c0e283362002e687c41785947241b3c2523bb9143c80ee82d50867259af4b47a332a8a0aa412f3258f7717826ed1e585af67f5712abe35fb533513d929087cbb364532da3340e377bb156f25c8ee3e2cabb986158eaefe7c3adb4f4a88771440947b1b0c1a34053ae

// // # return: 6a24aa21a9edcdcb2e39372f6650e4f9d730c34318cc4f0c8d2b9ba3ec2a8b9c74350f7b3044
















































use sha2::{Digest, Sha256};
use ripemd::{Ripemd160};
use std::io::{self, Write};

// Utility functions
fn hash160(data: &str) -> String {
    let binary = hex::decode(data).expect("Invalid hex string");
    let sha256 = Sha256::digest(&binary);
    let ripemd = Ripemd160::digest(&sha256);
    hex::encode(ripemd)
}

#[derive(Debug, Clone, PartialEq)]
enum ScriptType {
    P2PK,
    P2PKH,
    P2SH,
    P2MS,
    Return,
    Unknown,
}

#[derive(Debug, Clone)]
struct Script {
    asm: String,
    hex: String,
    script_type: ScriptType,
}

impl Script {
    fn new(data: &str) -> Self {
        let (hex, asm) = if data.chars().all(|c| c.is_ascii_hexdigit() || c.is_whitespace()) 
            && !data.contains(' ') {
            // It's hex
            let h = data.to_string();
            let a = Self::hex_to_asm(&h);
            (h, a)
        } else {
            // It's asm
            let a = data.to_string();
            let h = Self::asm_to_hex(&a);
            (h, a)
        };

        let script_type = Self::get_type(&asm);

        Script {
            asm,
            hex,
            script_type,
        }
    }

    fn hex_to_asm(hex: &str) -> String {
        let mut asm = Vec::new();
        let bytes: Vec<&str> = hex.as_bytes()
            .chunks(2)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();
        
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];
            let int_val = u8::from_str_radix(byte, 16).unwrap();

            if int_val > 0 && int_val < 0x4b {
                let data_len = int_val as usize;
                i += 1;
                let data: String = bytes[i..i + data_len].join("");
                asm.push(data);
                i += data_len;
            } else {
                asm.push(Opcodes::get_opcode(byte));
                i += 1;
            }
        }

        asm.join(" ")
    }

    fn asm_to_hex(asm: &str) -> String {
        let mut hex = String::new();
        let pieces: Vec<&str> = asm.split_whitespace().collect();

        for piece in pieces {
            if let Some(opcode_hex) = Opcodes::get_hex(piece) {
                hex.push_str(&opcode_hex);
            } else {
                // It's data, not an opcode
                let push = format!("{:02x}", piece.len() / 2);
                hex.push_str(&push);
                hex.push_str(piece);
            }
        }

        hex
    }

    fn get_type(asm: &str) -> ScriptType {
        let parts: Vec<&str> = asm.split_whitespace().collect();

        if parts.len() == 2 && parts[1] == "OP_CHECKSIG" {
            ScriptType::P2PK
        } else if parts.len() == 5 
            && parts[0] == "OP_DUP" 
            && parts[1] == "OP_HASH160" 
            && parts[3] == "OP_EQUALVERIFY" 
            && parts[4] == "OP_CHECKSIG" {
            ScriptType::P2PKH
        } else if parts.len() == 3 
            && parts[0] == "OP_HASH160" 
            && parts[2] == "OP_EQUAL" {
            ScriptType::P2SH
        } else if parts.len() >= 3
            && parts[0].starts_with("OP_")
            && parts[parts.len() - 2].starts_with("OP_")
            && parts[parts.len() - 1] == "OP_CHECKMULTISIG" {
            ScriptType::P2MS
        } else if parts.len() == 2 && parts[0] == "OP_RETURN" {
            ScriptType::Return
        } else {
            ScriptType::Unknown
        }
    }

    fn run(scripts: &[Script]) -> Result<Vec<String>, String> {
        // Check if P2SH
        if scripts.len() > 1 && scripts[1].script_type == ScriptType::P2SH {
            return Self::run_p2sh(scripts);
        }

        // Combine all scripts
        let mut script: Vec<String> = scripts
            .iter()
            .flat_map(|s| s.asm.split_whitespace().map(|x| x.to_string()))
            .collect();

        let mut stack: Vec<String> = Vec::new();

        while !script.is_empty() {
            let opcode = script.remove(0);
            
            if Opcodes::is_opcode(&opcode) {
                stack = Opcodes::execute(&opcode, stack)?;
            } else {
                stack.push(opcode);
            }
        }

        Ok(stack)
    }

    fn run_p2sh(scripts: &[Script]) -> Result<Vec<String>, String> {
        let unlocking = &scripts[0];
        let locking = &scripts[1];

        // Run unlocking script first
        let mut stack = Self::run(&[unlocking.clone()])?;
        let stack_copy = stack.clone();

        // Run combined script
        let combined = Script::new(&format!("{}{}", unlocking.hex, locking.hex));
        stack = Self::run(&[combined])?;

        // Validate primary script
        if Self::validate(&stack) {
            // Get redeem script from stack copy
            let mut stack_copy = stack_copy;
            let redeem_hex = stack_copy.pop().ok_or("No redeem script on stack")?;
            let redeem_script = Script::new(&redeem_hex);

            // Run secondary script
            let combined2 = Script::new(&format!("{}{}", unlocking.hex, redeem_script.hex));
            let stack2 = Self::run(&[combined2])?;

            return Ok(stack2);
        }

        Ok(stack)
    }

    fn validate(stack: &[String]) -> bool {
        if stack.is_empty() {
            return false;
        }

        let top = &stack[stack.len() - 1];
        
        if top == "OP_TRUE" {
            return true;
        }

        if top.starts_with("OP_") {
            if let Some(num_str) = top.strip_prefix("OP_") {
                if let Ok(num) = num_str.parse::<i32>() {
                    return num > 0;
                }
            }
        }

        false
    }
}

struct Opcodes;

impl Opcodes {
    fn get_opcode(hex: &str) -> String {
        match hex.to_lowercase().as_str() {
            "00" => "OP_0".to_string(),
            "51" => "OP_1".to_string(),
            "52" => "OP_2".to_string(),
            "53" => "OP_3".to_string(),
            "54" => "OP_4".to_string(),
            "55" => "OP_5".to_string(),
            "56" => "OP_6".to_string(),
            "57" => "OP_7".to_string(),
            "58" => "OP_8".to_string(),
            "59" => "OP_9".to_string(),
            "5a" => "OP_10".to_string(),
            "5b" => "OP_11".to_string(),
            "5c" => "OP_12".to_string(),
            "5d" => "OP_13".to_string(),
            "5e" => "OP_14".to_string(),
            "5f" => "OP_15".to_string(),
            "60" => "OP_16".to_string(),
            "6a" => "OP_RETURN".to_string(),
            "76" => "OP_DUP".to_string(),
            "87" => "OP_EQUAL".to_string(),
            "88" => "OP_EQUALVERIFY".to_string(),
            "ac" => "OP_CHECKSIG".to_string(),
            "ae" => "OP_CHECKMULTISIG".to_string(),
            "a9" => "OP_HASH160".to_string(),
            _ => hex.to_string(),
        }
    }

    fn get_hex(opcode: &str) -> Option<String> {
        match opcode {
            "OP_0" => Some("00".to_string()),
            "OP_1" => Some("51".to_string()),
            "OP_2" => Some("52".to_string()),
            "OP_3" => Some("53".to_string()),
            "OP_4" => Some("54".to_string()),
            "OP_5" => Some("55".to_string()),
            "OP_6" => Some("56".to_string()),
            "OP_7" => Some("57".to_string()),
            "OP_8" => Some("58".to_string()),
            "OP_9" => Some("59".to_string()),
            "OP_10" => Some("5a".to_string()),
            "OP_11" => Some("5b".to_string()),
            "OP_12" => Some("5c".to_string()),
            "OP_13" => Some("5d".to_string()),
            "OP_14" => Some("5e".to_string()),
            "OP_15" => Some("5f".to_string()),
            "OP_16" => Some("60".to_string()),
            "OP_RETURN" => Some("6a".to_string()),
            "OP_DUP" => Some("76".to_string()),
            "OP_EQUAL" => Some("87".to_string()),
            "OP_EQUALVERIFY" => Some("88".to_string()),
            "OP_CHECKSIG" => Some("ac".to_string()),
            "OP_CHECKMULTISIG" => Some("ae".to_string()),
            "OP_HASH160" => Some("a9".to_string()),
            _ => None,
        }
    }

    fn is_opcode(s: &str) -> bool {
        s.starts_with("OP_")
    }

    fn execute(opcode: &str, mut stack: Vec<String>) -> Result<Vec<String>, String> {
        match opcode {
            "OP_DUP" => {
                if stack.is_empty() {
                    return Err("Stack is empty, cannot duplicate".to_string());
                }
                let top = stack.last().unwrap().clone();
                stack.push(top);
                Ok(stack)
            }
            "OP_HASH160" => {
                if stack.is_empty() {
                    return Err("Stack is empty, cannot hash160".to_string());
                }
                let top = stack.pop().unwrap();
                let hashed = hash160(&top);
                stack.push(hashed);
                Ok(stack)
            }
            "OP_EQUAL" => {
                if stack.len() < 2 {
                    return Err("Not enough items on stack for OP_EQUAL".to_string());
                }
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                if a == b {
                    stack.push("OP_TRUE".to_string());
                    Ok(stack)
                } else {
                    Err(format!("Items not equal: {} != {}", a, b))
                }
            }
            "OP_EQUALVERIFY" => {
                if stack.len() < 2 {
                    return Err("Not enough items on stack for OP_EQUALVERIFY".to_string());
                }
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                if a == b {
                    Ok(stack)
                } else {
                    Err(format!("Items not equal: {} != {}", a, b))
                }
            }
            "OP_CHECKSIG" => {
                if stack.len() < 2 {
                    return Err("Not enough items on stack for OP_CHECKSIG".to_string());
                }
                let _pubkey = stack.pop().unwrap();
                let _signature = stack.pop().unwrap();
                // Simplified - not actually validating
                stack.push("OP_TRUE".to_string());
                Ok(stack)
            }
            "OP_CHECKMULTISIG" => {
                if stack.is_empty() {
                    return Err("Stack empty for OP_CHECKMULTISIG".to_string());
                }
                let m_str = stack.pop().unwrap();
                let m = m_str.strip_prefix("OP_")
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("Invalid m value")?;
                
                if stack.len() < m {
                    return Err("Not enough signatures on stack".to_string());
                }
                for _ in 0..m {
                    stack.pop();
                }

                if stack.is_empty() {
                    return Err("Stack empty getting n value".to_string());
                }
                let n_str = stack.pop().unwrap();
                let n = n_str.strip_prefix("OP_")
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("Invalid n value")?;
                
                if stack.len() < n + 1 {
                    return Err("Not enough public keys on stack".to_string());
                }
                for _ in 0..n + 1 {
                    stack.pop();
                }

                stack.push("OP_TRUE".to_string());
                Ok(stack)
            }
            "OP_RETURN" => {
                Err("Script is invalid. (OP_RETURN always invalidates a script.)".to_string())
            }
            _ => Ok(stack),
        }
    }
}

fn main() {
    println!("Bit_Check");
    println!("==========================\n");

    print!("Locking Script: ");
    io::stdout().flush().unwrap();
    let mut locking_input = String::new();
    io::stdin().read_line(&mut locking_input).unwrap();
    let locking_script = Script::new(locking_input.trim());
    println!("Type: {:?}\n", locking_script.script_type);

    print!("Unlocking Script: ");
    io::stdout().flush().unwrap();
    let mut unlocking_input = String::new();
    io::stdin().read_line(&mut unlocking_input).unwrap();
    let unlocking_script = Script::new(unlocking_input.trim());

    println!("\nLocking Script: {}", locking_script.asm);
    println!("Unlocking Script: {}", unlocking_script.asm);
    println!();

    print!("Run this script? (y/n): ");
    io::stdout().flush().unwrap();
    let mut yn = String::new();
    io::stdin().read_line(&mut yn).unwrap();

    if yn.trim() == "y" || yn.trim().is_empty() {
        match Script::run(&[unlocking_script, locking_script]) {
            Ok(stack) => {
                println!("\nFinal Stack: {:?}", stack);
                if Script::validate(&stack) {
                    println!("\n✓ This is a valid script!");
                } else {
                    println!("\n✗ This is not a valid script.");
                }
            }
            Err(e) => {
                println!("\n✗ Script execution failed: {}", e);
            }
        }
    }
}

// scriptPubKey hex: 1976a9146291ad5107bf1ab687dc744cc9d082aa9522eff088ac
// scriptPubKey asm: OP_DUP OP_HASH160 6291ad5107bf1ab687dc744cc9d082aa9522eff0 OP_EQUALVERIFY 

// scriptSig hex: 6b483045022100c9b46687338c296533a8cd914b9e7e7930535d8c614856ebc7ecaf69fcc3088d0220221409c6d8912d703c32846c975f80c56fd6079e4a03c3311089d27fab730edd012103d031d8d65ef3d737f4f231c11d29c0d037717850402efc55c3544bec1217efd2
// scriptSig asm: 3045022100c9b46687338c296533a8cd914b9e7e7930535d8c614856ebc7ecaf69fcc3088d0220221409c6d8912d703c32846c975f80c56fd6079e4a03c3311089d27fab730edd01 03d031d8d65ef3d737f4f231c11d29c0d037717850402efc55c3544bec1217efd2