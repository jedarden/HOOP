import { useRef, useEffect } from 'react';
import { TranscriptData, TranscriptWord } from '../atoms';

interface TranscriptViewProps {
  transcript: TranscriptData;
  currentTime: number;
  onWordClick?: (time: number) => void;
}

export default function TranscriptView({ transcript, currentTime, onWordClick }: TranscriptViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const activeWordRef = useRef<HTMLSpanElement>(null);

  const handleWordClick = (word: TranscriptWord) => {
    onWordClick?.(word.start);
  };

  const getCurrentWordIndex = (): number => {
    for (let i = 0; i < transcript.words.length; i++) {
      if (currentTime >= transcript.words[i].start && currentTime <= transcript.words[i].end) {
        return i;
      }
    }
    return -1;
  };

  const currentWordIndex = getCurrentWordIndex();

  // Auto-scroll to keep the active word visible
  useEffect(() => {
    if (activeWordRef.current && containerRef.current) {
      const container = containerRef.current;
      const word = activeWordRef.current;

      const containerRect = container.getBoundingClientRect();
      const wordRect = word.getBoundingClientRect();

      // Scroll if the active word is outside the visible middle third
      if (wordRect.top < containerRect.top || wordRect.bottom > containerRect.bottom) {
        word.scrollIntoView({ block: 'center', behavior: 'smooth' });
      }
    }
  }, [currentWordIndex]);

  return (
    <div className="transcript-view" ref={containerRef}>
      <div className="transcript-words">
        {transcript.words.map((word, index) => {
          const isActive = index === currentWordIndex;
          const isPast = index < currentWordIndex;
          return (
            <span
              key={index}
              ref={isActive ? activeWordRef : undefined}
              className={`transcript-word ${isActive ? 'active' : ''} ${isPast ? 'past' : ''}`}
              onClick={() => handleWordClick(word)}
            >
              {word.word}
            </span>
          );
        })}
      </div>
    </div>
  );
}
