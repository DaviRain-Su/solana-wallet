import Editor from '@monaco-editor/react';

interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
}

function CodeEditor({ value, onChange }: CodeEditorProps) {
  return (
    <Editor
      height="500px"
      defaultLanguage="rust"
      theme="vs-dark"
      value={value}
      onChange={(value) => onChange(value || '')}
      options={{
        minimap: { enabled: false },
        fontSize: 14,
        wordWrap: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
      }}
    />
  );
}

export default CodeEditor;