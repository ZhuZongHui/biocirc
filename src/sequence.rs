use super::codon;

use std::ops::Add;
use std::slice::Chunks;
use std::collections::HashMap;
use std::{cmp::PartialEq, fmt};
/// 用于生物 CircRNA以及 基础生物序列操作
/// 主要是尝试输出来让自己熟练

/// 首先是生物序列对象

const DNA_BASE_PAIRING: [(char, char); 4] = [('A', 'T'), ('G', 'C'), ('T', 'A'), ('C', 'G')];

const RNA_BASE_PAIRING: [(char, char); 4] = [('A', 'U'), ('G', 'C'), ('U', 'A'), ('C', 'G')];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BioType {
    Dna,
    Rna,
    Protein,
}

// fmt trait 用于错误处理中实现格式化
impl fmt::Display for BioType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BioType::Dna => write!(f, "DNA"),
            BioType::Protein => write!(f, "PROTEIN"),
            BioType::Rna => write!(f, "RNA"),
        }
    }
}

#[derive(Debug, Clone)]

// 重新设计？ 
// 添加一个密码子表，并赋予一个默认值
// 然后缓存对应的密码子表？ 如何实现

pub struct Sequence {
    pub biotype: BioType,
    pub seq: String,
}

impl Sequence {
    pub fn new(biotype: BioType, seq: String) -> Self {
        Sequence { biotype, seq }
    }
    /// 获取对应索引并返回字符对象，不存在修改
    pub fn index(&self, index: usize) -> char {
        self.seq[index..=index].chars().next().unwrap()
    }
    /// 添加字符
    pub fn push(&mut self, ch: char) -> () {
        self.seq.push(ch);
    }
    /// 用于修改字符串中某位置的某值，如果需要大片段替换请直接操作字符串，因为可能会非常慢
    pub fn change(&mut self, index: usize, ch: char) {
        let mut replaced = String::with_capacity(self.seq.len());
        for (i, c) in self.seq.char_indices() {
            if i == index {
                replaced.push(ch);
            } else {
                replaced.push(c);
            }
        }
        self.seq = replaced;
    }
    /// 返回长度
    pub fn len(&self) -> usize {
        self.seq.len()
    }

    /// 计数
    pub fn count(&self, string: &str) -> usize {
        self.seq.matches(string).count()
    }
}

/// 两个序列可以直接使用加号
impl Add for Sequence {
    type Output = Sequence;
    fn add(self, rhs: Self) -> Self::Output {
        if self.biotype == rhs.biotype {
            Sequence {
                biotype: self.biotype,
                seq: self.seq + &rhs.seq,
            }
        } else {
            panic!("类型错误{}加到{}", self.biotype, rhs.biotype);
        }
    }
}

impl<T: Into<String>> Add<T> for Sequence {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Sequence {
            biotype: self.biotype.clone(),
            seq: self.seq + &rhs.into(),
        }
    }
}


/// 直接判断两个序列是否相等，虽然感觉没有用的功能
impl PartialEq for Sequence {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq
    }
}

impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seq: String = self.seq.clone();
        let chunks: Chunks<u8> = seq.as_bytes().chunks(80);
        let fmt_seq: String = chunks
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<&str>>()
            .join("\n");
        write!(
            f,
            "Bio Sequence Type is :{}\nSequence:\n{}",
            self.biotype, fmt_seq
        )
    }
}

impl Sequence {
    /// 将序列翻译为蛋白质
    pub fn translate(&self) -> Result<Sequence, String> {
        let codon_table: HashMap<&str, &str> = codon::CODON_TABLE.iter().cloned().collect();
        let seq: String = if self.biotype == BioType::Dna {
            Self::transcribe(&self).unwrap().seq
        } else {
            self.seq.clone().to_uppercase()
        };

        match self.biotype {
            BioType::Dna | BioType::Rna => {
                let seq_chars: Vec<char> = seq.chars().collect();
                let mut chunks = seq_chars.chunks(3);
                let mut protein_seq = String::new();
                while let Some(chunk) = chunks.next() {
                    if chunk.len() < 3 {
                        break;
                    }
                    let chunk_str: String = chunk.iter().collect();
                    let coden = codon_table[&chunk_str[..]];
                    protein_seq.push_str(coden);
                    if coden == "*" {
                        break;
                    } // 如果遇到终止密码子则提前返回
                }

                Ok(Sequence::new(BioType::Protein, seq))
            }
            BioType::Protein => Err(format!("你不能翻译一段{}序列", BioType::Protein)),
        }
    }

    /// 将DNA序列转录为RNA
    pub fn transcribe(&self) -> Result<Sequence, String> {
        match self.biotype {
            BioType::Dna => {
                let seq: String = self.seq.clone().to_uppercase().replace("T", "U");
                Ok(Sequence {
                    biotype: BioType::Rna,
                    seq: seq,
                })
            }
            BioType::Protein | BioType::Rna => Err(format!("你不能转录一段{}序列", self.biotype)),
        }
    }

    pub fn back_transcription(&self) -> Result<Sequence, String> {
        match self.biotype {
            BioType::Rna => {
                let seq = self.seq.clone().to_uppercase().replace("U", "T");
                Ok(Sequence {
                    biotype: BioType::Rna,
                    seq: seq,
                })
            }
            BioType::Protein | BioType::Dna => Err(format!("你不能逆转录一段{}序列", self.biotype)),
        }
    }

    /// 获得一段序列的互补序列 DNA 或 RNA
    pub fn complementary(&self) -> Result<Sequence, String> {
        match self.biotype {
            BioType::Dna => {
                let pairing_table: HashMap<char, char> = DNA_BASE_PAIRING.iter().cloned().collect();
                let seq = self.seq.clone().to_uppercase();
                let mut complement = String::with_capacity(seq.len());

                for base in seq.chars() {
                    match pairing_table.get(&base) {
                        Some(&complement_base) => complement.push(complement_base),
                        None => return Err(format!("Invalid DNA base: {}", base)),
                    }
                }
                Ok(Sequence::new(self.biotype.clone(), complement))
            }
            BioType::Rna => {
                let pairing_table: HashMap<char, char> = RNA_BASE_PAIRING.iter().cloned().collect();

                let seq = self.seq.clone().to_uppercase();
                let mut complement = String::with_capacity(seq.len());

                for base in seq.chars() {
                    match pairing_table.get(&base) {
                        Some(&complement_base) => complement.push(complement_base),
                        None => return Err(format!("Invalid RNA base: {}", base)),
                    }
                }
                Ok(Sequence::new(self.biotype.clone(), complement))
            }
            BioType::Protein => Err(format!("你不能反向互补一段 {} 序列", self.biotype)),
        }
    }

    /// 获得一段序列的反向互补序列 DNA 或 RNA
    pub fn reverse_complementary(&self) -> Result<Sequence, String> {
        let mut sequence = Self::complementary(&self)?;
        sequence.seq = sequence.seq.chars().rev().collect();
        Ok(sequence)
    }
}