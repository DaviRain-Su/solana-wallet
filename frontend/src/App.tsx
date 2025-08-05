import { useState } from 'react'
import CodeEditor from './components/CodeEditor'
import OutputPanel from './components/OutputPanel'
import './App.css'

function App() {
  const [code, setCode] = useState(`fn main() {
    println!("Hello, Rust!");
}`);
  const [output, setOutput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runCode = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await fetch('/api/execute', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ code, mode: 'debug' }),
      });
      
      const data = await response.json();
      
      if (data.success) {
        setOutput(data.output);
      } else {
        setError(data.error || 'Failed to execute code');
      }
    } catch (err) {
      setError('Failed to connect to server');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>Rust Book Online</h1>
        <p>Learn Rust interactively</p>
      </header>
      
      <main className="app-main">
        <div className="editor-section">
          <div className="toolbar">
            <button 
              onClick={runCode} 
              disabled={isLoading}
              className="run-button"
            >
              {isLoading ? 'Running...' : 'Run Code'}
            </button>
          </div>
          
          <CodeEditor 
            value={code} 
            onChange={setCode}
          />
        </div>
        
        <div className="output-section">
          <OutputPanel 
            output={output} 
            error={error}
          />
        </div>
      </main>
    </div>
  )
}

export default App