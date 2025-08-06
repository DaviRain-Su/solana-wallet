export interface Course {
  id: string;
  title: string;
  description: string;
  language: 'rust' | 'solana';
  chapters: Chapter[];
}

export interface Chapter {
  id: string;
  title: string;
  lessons: Lesson[];
}

export interface Lesson {
  id: string;
  title: string;
  content: string;
  initialCode: string;
  solution?: string;
  hints?: string[];
}

export const courses: Course[] = [
  {
    id: 'rust-basics',
    title: 'Rust基础',
    description: '学习Rust编程语言的基础知识',
    language: 'rust',
    chapters: [
      {
        id: 'getting-started',
        title: '入门',
        lessons: [
          {
            id: 'hello-world',
            title: 'Hello, World!',
            content: `# Hello, World!

欢迎来到Rust编程世界！让我们从经典的"Hello, World!"程序开始。

在Rust中，我们使用\`println!\`宏来打印文本到控制台。

## 任务
修改下面的代码，让它打印出"Hello, Rust!"`,
            initialCode: `fn main() {
    println!("Hello, World!");
}`,
            solution: `fn main() {
    println!("Hello, Rust!");
}`,
            hints: ['修改println!宏中的文本']
          },
          {
            id: 'variables',
            title: '变量和可变性',
            content: `# 变量和可变性

在Rust中，变量默认是不可变的。这是Rust许多安全保证的一部分。

## 任务
1. 创建一个可变变量 \`x\`，初始值为5
2. 将 \`x\` 的值改为10
3. 打印 \`x\` 的值`,
            initialCode: `fn main() {
    // 在这里创建变量
    
    // 修改变量的值
    
    // 打印变量
}`,
            solution: `fn main() {
    let mut x = 5;
    x = 10;
    println!("x的值是: {}", x);
}`
          }
        ]
      },
      {
        id: 'ownership',
        title: '所有权',
        lessons: [
          {
            id: 'ownership-basics',
            title: '所有权基础',
            content: `# 所有权

所有权是Rust最独特的特性，它让Rust无需垃圾回收器就能保证内存安全。

## 所有权规则
1. Rust中的每个值都有一个所有者
2. 值在任一时刻有且只有一个所有者
3. 当所有者离开作用域时，这个值将被丢弃

## 任务
理解下面代码中的所有权转移`,
            initialCode: `fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1的所有权转移给了s2
    
    // 尝试使用s1会导致编译错误
    // println!("{}", s1);
    
    println!("{}", s2);
}`
          }
        ]
      }
    ]
  },
  {
    id: 'solana-basics',
    title: 'Solana开发基础',
    description: '学习Solana区块链开发',
    language: 'solana',
    chapters: [
      {
        id: 'solana-intro',
        title: 'Solana简介',
        lessons: [
          {
            id: 'what-is-solana',
            title: '什么是Solana',
            content: `# Solana简介

Solana是一个高性能的区块链平台，专为去中心化应用和加密货币而设计。

## 主要特点
- 高吞吐量：每秒可处理数千笔交易
- 低成本：交易费用极低
- 快速确认：400毫秒的出块时间
- 使用Rust编写智能合约

## 任务
在下面的代码中，我们将连接到Solana网络并获取账户信息。`,
            initialCode: `// 这是一个TypeScript示例
// 实际运行需要Node.js环境

import { Connection, PublicKey } from "@solana/web3.js";

async function main() {
    // 连接到Solana devnet
    const connection = new Connection("https://api.devnet.solana.com");
    
    // 创建一个公钥
    const publicKey = new PublicKey("11111111111111111111111111111111");
    
    // 获取账户余额
    const balance = await connection.getBalance(publicKey);
    console.log(\`余额: \${balance} lamports\`);
}

main();`
          }
        ]
      }
    ]
  }
];