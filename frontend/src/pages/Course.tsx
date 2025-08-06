import { useState, useEffect } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';
import CodeEditor from '../components/CodeEditor';
import OutputPanel from '../components/OutputPanel';
import { courses } from '../../../shared/courses';
import './Course.css';

function Course() {
  const { courseId, chapterId, lessonId } = useParams();
  const navigate = useNavigate();
  
  const [code, setCode] = useState('');
  const [output, setOutput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showHint, setShowHint] = useState(false);
  const [currentHintIndex, setCurrentHintIndex] = useState(0);

  const course = courses.find(c => c.id === courseId);
  const chapter = course?.chapters.find(ch => ch.id === chapterId);
  const lesson = chapter?.lessons.find(l => l.id === lessonId);

  useEffect(() => {
    if (lesson) {
      setCode(lesson.initialCode);
      setOutput('');
      setError(null);
      setShowHint(false);
      setCurrentHintIndex(0);
    }
  }, [lesson]);

  const runCode = async () => {
    if (!course) return;
    
    setIsLoading(true);
    setError(null);
    
    try {
      // Only run Rust code through the API
      if (course.language === 'rust') {
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
      } else {
        // For Solana/JS code, just show a message
        setOutput('Solana代码示例仅供参考。实际运行需要配置Node.js环境和Solana依赖。');
      }
    } catch (err) {
      setError('Failed to connect to server');
    } finally {
      setIsLoading(false);
    }
  };

  const showNextHint = () => {
    if (lesson?.hints && currentHintIndex < lesson.hints.length - 1) {
      setCurrentHintIndex(currentHintIndex + 1);
    }
    setShowHint(true);
  };

  const showSolution = () => {
    if (lesson?.solution) {
      setCode(lesson.solution);
    }
  };

  if (!course || !chapter || !lesson) {
    return (
      <div className="course-error">
        <h2>课程未找到</h2>
        <Link to="/">返回首页</Link>
      </div>
    );
  }

  const currentChapterIndex = course.chapters.indexOf(chapter);
  const currentLessonIndex = chapter.lessons.indexOf(lesson);
  
  const nextLesson = chapter.lessons[currentLessonIndex + 1] ||
    course.chapters[currentChapterIndex + 1]?.lessons[0];
  
  const prevLesson = chapter.lessons[currentLessonIndex - 1] ||
    course.chapters[currentChapterIndex - 1]?.lessons[
      course.chapters[currentChapterIndex - 1].lessons.length - 1
    ];

  return (
    <div className="course-container">
      <aside className="course-sidebar">
        <Link to="/" className="back-link">← 返回首页</Link>
        <h2>{course.title}</h2>
        {course.chapters.map(ch => (
          <div key={ch.id} className="chapter-section">
            <h3>{ch.title}</h3>
            <ul>
              {ch.lessons.map(l => (
                <li key={l.id}>
                  <Link
                    to={`/course/${course.id}/${ch.id}/${l.id}`}
                    className={l.id === lessonId ? 'active' : ''}
                  >
                    {l.title}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </aside>

      <main className="course-main">
        <div className="lesson-content">
          <ReactMarkdown>{lesson.content}</ReactMarkdown>
        </div>

        <div className="code-section">
          <div className="toolbar">
            <button onClick={runCode} disabled={isLoading} className="run-button">
              {isLoading ? '运行中...' : '运行代码'}
            </button>
            {lesson.hints && (
              <button onClick={showNextHint} className="hint-button">
                提示 ({currentHintIndex + 1}/{lesson.hints.length})
              </button>
            )}
            {lesson.solution && (
              <button onClick={showSolution} className="solution-button">
                查看答案
              </button>
            )}
          </div>

          {showHint && lesson.hints && (
            <div className="hint-box">
              💡 {lesson.hints[currentHintIndex]}
            </div>
          )}

          <CodeEditor value={code} onChange={setCode} />
        </div>

        <div className="output-section">
          <OutputPanel output={output} error={error} />
        </div>

        <div className="navigation-buttons">
          {prevLesson && (
            <button
              onClick={() => {
                const prevCh = course.chapters.find(ch => 
                  ch.lessons.includes(prevLesson)
                );
                if (prevCh) {
                  navigate(`/course/${course.id}/${prevCh.id}/${prevLesson.id}`);
                }
              }}
              className="nav-button prev"
            >
              ← 上一课
            </button>
          )}
          {nextLesson && (
            <button
              onClick={() => {
                const nextCh = course.chapters.find(ch => 
                  ch.lessons.includes(nextLesson)
                );
                if (nextCh) {
                  navigate(`/course/${course.id}/${nextCh.id}/${nextLesson.id}`);
                }
              }}
              className="nav-button next"
            >
              下一课 →
            </button>
          )}
        </div>
      </main>
    </div>
  );
}

export default Course;