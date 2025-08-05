use std::collections::{HashMap, VecDeque};
use rand::{distr::{weighted::WeightedIndex, Distribution}, rng, Rng};
use tokio::sync::mpsc;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Init,
    InWord,
    InWhitespace
}

fn tokenize(inp: String) -> Vec<String> {
    let mut out = vec![];
    let mut buffer = String::new();
    let mut state = State::Init;
    let special_chars = "`~!@#$%^&*()-=_+[]\\{}|;\":',./<>?’“‘”–—".chars().collect::<Vec<_>>();

    for c in inp.chars() {
        if special_chars.contains(&c) {
            continue
        }
        if c.is_whitespace() {
            if state == State::InWord {
                if buffer.len() > 0 { out.push(buffer.clone().to_lowercase()); }
                buffer.clear();
            }
            state = State::InWhitespace
        } else if !c.is_ascii() {
            if state == State::InWord {
                if buffer.len() > 0 { out.push(buffer.clone().to_lowercase()); }
                buffer.clear();
            }
            out.push(c.to_string())
        } else {
            buffer.push(c);
            state = State::InWord
        }
    }

    if state == State::InWord {
        if buffer.len() > 0 { out.push(buffer.to_lowercase()); }
    }

    //let words = ["a", "of", "the", "for", "to", "and", "an", "so"];

    //out.retain(|x| !words.contains(&x.as_str()));

    out
}

struct Markov {
    degree: u8,
    words: Vec<String>,
    mappings: HashMap<VecDeque<u32>, Vec<(u32, u32)>>
}

impl Markov {
    pub fn new(degree: u8) -> Markov {
        Markov { degree, words: vec![], mappings: HashMap::new() }
    }

    pub fn train(&mut self, corpus: &[String], weight: u32) {
        let mut cache = HashMap::<String, u32>::new();
        let mut prev = VecDeque::new();

        for word in corpus {
            let idx = cache.get(word);
            let idx = match idx {
                Some(v) => *v,
                None => {
                    if let Some((i, _)) = self.words.iter().enumerate().find(|x| x.1 == word) {
                        cache.insert(word.to_owned(), i as u32);
                        i as u32
                    } else {
                        self.words.push(word.clone());
                        (self.words.len() - 1) as u32
                    }
                }
            };


            if prev.len() > 0 {
                // find existing
                if let Some(v) = self.mappings.get_mut(&prev) {
                    if let Some((_, n)) = v.iter_mut().find(|(a, _)| *a == idx) {
                        *n += weight
                    } else {
                        v.push((idx, weight));
                    }
                } else {
                    self.mappings.insert(prev.clone(), vec![(idx, weight)]);
                }
            }

            prev.push_back(idx);
            if prev.len() >= self.degree as usize {
                prev.pop_front();
            }
        }
    }

    pub fn infer(&self, limit: usize) -> Vec<String> {
        let mut out = vec![];
        let mut prev = VecDeque::new();

        let mut r = rng();
        let random = r.random_range(0..self.words.len());
        out.push(self.words[random].clone());
        prev.push_back(random as u32);

        for _ in 1..limit {
            let mut trimmed = prev.clone();
            for _ in 0..self.degree {
                if let Some(next) = self.mappings.iter().find(|x| x.0.iter().rev().zip(trimmed.iter().rev()).all(|(a, b)| a == b)).map(|(_, b)| b) {
                    let dist = WeightedIndex::new(next.iter().map(|x| x.1)).unwrap();
                    let n = next[dist.sample(&mut r)].0;
                    prev.push_back(n);
                    if prev.len() >= self.degree as usize {
                        prev.pop_front();
                    }
                    out.push(self.words[n as usize].clone());
                    break
                }

                trimmed.pop_front();
            }
        }
        out
    }
}

#[tokio::main]
async fn main() -> ! {
    let inp = std::fs::read_to_string("/home/foo/Downloads/art-of-war-2.txt").unwrap();
    let mut chain = Markov::new(2);
    let tokens = tokenize(inp);
    chain.train(&tokens, 1);
    
    let (tx, mut rx) = mpsc::channel::<String>(32);

    tokio::spawn(async move {
        let mut sock = zeromq::RepSocket::new();
        sock.connect("tcp://127.0.0.1:5555").await.expect("failed to connect to socket");
        loop {
            let v = sock.recv().await.unwrap().into_vec().iter().flatten().cloned().collect::<Vec<_>>();
            if v == "generate".as_bytes() {
                sock.send(rx.recv().await.unwrap().into()).await.unwrap();
            }
        }
    });

    loop {
        let out = chain.infer(3).to_vec();
        if !tokens.chunks(out.len()).any(|x| x == out) {
            tx.send(out.join(" ")).await.unwrap();
        }
    }
}

