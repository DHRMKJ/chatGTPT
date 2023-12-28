pub type Result<T> = std::result::Result<T, ()>;

#[derive(Debug)]
pub enum Message {
    ClientConnected{
        stream: std::sync::Arc<tokio::net::TcpStream>,
        addr: std::net::SocketAddr
    },
    ClientDisconnected (std::net::SocketAddr),
    NewMessage{message: Vec<u8>, addr: std::net::SocketAddr},
}

use std::collections::HashMap;

#[derive(Default, Debug)]
struct Node {
    children: HashMap<char, Node>,
    is_end_of_word: bool
}

#[derive(Debug)]
pub struct Trie {
    node: Node,
}

impl Trie {
    fn new() -> Trie {
        Trie {
            node: Node::default()
        }
    }

    fn insert(&mut self, word: &str) {
        let mut curr = &mut self.node;
        for c in word.chars() {
            curr = curr.children.entry(c).or_insert(Node::default());  
        }
        curr.is_end_of_word = true;
    }

    fn contains(&self, words: &str) -> bool {
        let mut curr = &self.node;
        let words = words.to_lowercase();

        for c in words.chars() {
            if let Some(next_node) = curr.children.get(&c) {
                curr = next_node;
                if curr.is_end_of_word {
                    return true;
                }
            }else {
                curr = &self.node;
            }
        }
        false
    }
}


pub fn fill_banned_words() -> Trie {
    let words: Vec<&str> = include_str!("data/en.txt").lines().collect();
    let mut trie = Trie::new();
    words.iter().for_each(|word| trie.insert(word));
    trie
}


#[derive(Debug)]
pub struct Client {
    pub stream: std::sync::Arc<tokio::net::TcpStream>,
    pub addr: std::net::SocketAddr,
    pub warnings: u8
}


pub fn validate_message(message: &Vec<u8>, bad_words: &Trie) -> bool {
    let sent = std::str::from_utf8(&message);
    let mut bad_boy = false;
    match sent {
        Ok(sent) => bad_boy |= bad_words.contains(sent),
        Err(_e) => { return false; }
    }
    bad_boy
}
