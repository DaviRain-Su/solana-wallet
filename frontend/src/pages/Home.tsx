import { Link } from 'react-router-dom';
import './Home.css';

function Home() {
  return (
    <div className="home">
      <div className="hero">
        <h1>学习 Rust & Solana</h1>
        <p>通过交互式编程练习，掌握现代编程技术</p>
      </div>

      <div className="courses">
        <div className="course-card">
          <h2>🦀 Rust 编程语言</h2>
          <p>学习系统编程语言Rust，掌握内存安全和并发编程</p>
          <ul>
            <li>零成本抽象</li>
            <li>内存安全</li>
            <li>并发无忧</li>
            <li>实用高效</li>
          </ul>
          <Link to="/course/rust-basics" className="start-button">
            开始学习 Rust
          </Link>
        </div>

        <div className="course-card">
          <h2>⚡ Solana 区块链开发</h2>
          <p>构建高性能的去中心化应用</p>
          <ul>
            <li>高吞吐量</li>
            <li>低交易成本</li>
            <li>快速确认</li>
            <li>Rust智能合约</li>
          </ul>
          <Link to="/course/solana-basics" className="start-button">
            开始学习 Solana
          </Link>
        </div>
      </div>

      <div className="playground-link">
        <Link to="/playground" className="playground-button">
          🚀 直接进入代码练习场
        </Link>
      </div>
    </div>
  );
}

export default Home;