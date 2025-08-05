interface OutputPanelProps {
  output: string;
  error: string | null;
}

function OutputPanel({ output, error }: OutputPanelProps) {
  return (
    <div className="output-panel">
      <h3>Output</h3>
      <div className="output-content">
        {error ? (
          <pre className="error-output">{error}</pre>
        ) : (
          <pre className="success-output">{output || 'Click "Run Code" to see output'}</pre>
        )}
      </div>
    </div>
  );
}

export default OutputPanel;