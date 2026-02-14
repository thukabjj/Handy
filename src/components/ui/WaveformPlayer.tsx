import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Loader2, Pause, Play, SkipBack, SkipForward } from "lucide-react";
import WaveSurfer from "wavesurfer.js";

import { SpeedControl } from "./SpeedControl";

interface WaveformPlayerProps {
  src: string;
  className?: string;
}

const SKIP_SHORT = 5;
const SKIP_LONG = 30;

function formatTime(seconds: number): string {
  if (!isFinite(seconds) || seconds < 0) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export const WaveformPlayer: React.FC<WaveformPlayerProps> = ({
  src,
  className = "",
}) => {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement>(null);
  const waveformRef = useRef<HTMLDivElement>(null);
  const wavesurferRef = useRef<WaveSurfer | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [speed, setSpeed] = useState(1);

  // Create and manage WaveSurfer instance
  useEffect(() => {
    if (!waveformRef.current || !src) return;

    // Destroy previous instance
    if (wavesurferRef.current) {
      wavesurferRef.current.destroy();
      wavesurferRef.current = null;
    }

    setIsReady(false);
    setIsPlaying(false);
    setCurrentTime(0);
    setDuration(0);

    const ws = WaveSurfer.create({
      container: waveformRef.current,
      waveColor: "#64748B",
      progressColor: "#3B82F6",
      cursorColor: "#F97316",
      height: 80,
      barWidth: 2,
      barGap: 1,
      barRadius: 2,
      normalize: true,
      backend: "WebAudio",
    });

    wavesurferRef.current = ws;

    ws.on("ready", () => {
      setIsReady(true);
      setDuration(ws.getDuration());
      ws.setPlaybackRate(speed);
    });

    ws.on("play", () => setIsPlaying(true));
    ws.on("pause", () => setIsPlaying(false));
    ws.on("finish", () => setIsPlaying(false));

    ws.on("timeupdate", (time: number) => {
      setCurrentTime(time);
    });

    ws.on("error", (error: Error) => {
      console.error("WaveSurfer error:", error);
      setIsReady(false);
    });

    ws.load(src);

    return () => {
      ws.destroy();
      wavesurferRef.current = null;
    };
    // We intentionally only re-create when src changes. Speed is handled separately.
  }, [src]);

  // Sync playback speed
  useEffect(() => {
    const ws = wavesurferRef.current;
    if (ws && isReady) {
      ws.setPlaybackRate(speed);
    }
  }, [speed, isReady]);

  const togglePlayPause = useCallback(() => {
    const ws = wavesurferRef.current;
    if (!ws || !isReady) return;
    ws.playPause();
  }, [isReady]);

  const skip = useCallback(
    (seconds: number) => {
      const ws = wavesurferRef.current;
      if (!ws || !isReady) return;
      const newTime = Math.max(0, Math.min(ws.getCurrentTime() + seconds, duration));
      ws.seekTo(newTime / duration);
    },
    [isReady, duration],
  );

  const handleSpeedChange = useCallback((newSpeed: number) => {
    setSpeed(newSpeed);
  }, []);

  // Cycle speed up/down for keyboard shortcuts
  const cycleSpeed = useCallback(
    (direction: "up" | "down") => {
      const presets = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2];
      const currentIndex = presets.indexOf(speed);
      if (currentIndex === -1) {
        setSpeed(1);
        return;
      }
      if (direction === "up" && currentIndex < presets.length - 1) {
        setSpeed(presets[currentIndex + 1]);
      } else if (direction === "down" && currentIndex > 0) {
        setSpeed(presets[currentIndex - 1]);
      }
    },
    [speed],
  );

  // Keyboard shortcuts
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      // Only handle when this component is focused
      if (!containerRef.current?.contains(event.target as Node)) return;

      // Ignore if user is typing in an input inside this component (unlikely, but safe)
      const tagName = (event.target as HTMLElement).tagName;
      if (tagName === "INPUT" || tagName === "TEXTAREA") return;

      switch (event.key) {
        case " ":
          event.preventDefault();
          togglePlayPause();
          break;

        case "ArrowLeft":
          event.preventDefault();
          skip(event.shiftKey ? -SKIP_LONG : -SKIP_SHORT);
          break;

        case "ArrowRight":
          event.preventDefault();
          skip(event.shiftKey ? SKIP_LONG : SKIP_SHORT);
          break;

        case "ArrowUp":
          event.preventDefault();
          cycleSpeed("up");
          break;

        case "ArrowDown":
          event.preventDefault();
          cycleSpeed("down");
          break;
      }
    },
    [togglePlayPause, skip, cycleSpeed],
  );

  return (
    <div
      ref={containerRef}
      className={`flex flex-col gap-2 ${className}`}
      tabIndex={0}
      onKeyDown={handleKeyDown}
      role="region"
      aria-label="Audio player"
    >
      {/* Waveform display */}
      <div className="relative rounded-lg overflow-hidden bg-mid-gray/5 border border-mid-gray/20">
        <div ref={waveformRef} className="w-full" />

        {/* Loading overlay */}
        {!isReady && (
          <div className="absolute inset-0 flex items-center justify-center bg-background/60">
            <Loader2
              className="w-6 h-6 text-primary-light animate-spin"
              aria-hidden="true"
            />
            <span className="sr-only">{t("common.loading")}</span>
          </div>
        )}
      </div>

      {/* Controls row */}
      <div className="flex items-center gap-2">
        {/* Skip back */}
        <button
          type="button"
          onClick={() => skip(-SKIP_SHORT)}
          disabled={!isReady}
          className="p-1 text-text-secondary hover:text-primary-light transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-light rounded"
          aria-label="Skip back 5 seconds"
        >
          <SkipBack className="w-4 h-4" />
        </button>

        {/* Play/Pause */}
        <button
          type="button"
          onClick={togglePlayPause}
          disabled={!isReady}
          className="p-1.5 rounded-full bg-primary-light/10 text-primary-light hover:bg-primary-light/20 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-light"
          aria-label={isPlaying ? "Pause" : "Play"}
        >
          {isPlaying ? (
            <Pause className="w-4 h-4" fill="currentColor" />
          ) : (
            <Play className="w-4 h-4" fill="currentColor" />
          )}
        </button>

        {/* Skip forward */}
        <button
          type="button"
          onClick={() => skip(SKIP_SHORT)}
          disabled={!isReady}
          className="p-1 text-text-secondary hover:text-primary-light transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-light rounded"
          aria-label="Skip forward 5 seconds"
        >
          <SkipForward className="w-4 h-4" />
        </button>

        {/* Time display */}
        <span className="text-xs text-text-secondary tabular-nums min-w-[80px] text-center select-none">
          {formatTime(currentTime)} / {formatTime(duration)}
        </span>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Speed control */}
        <SpeedControl speed={speed} onChange={handleSpeedChange} />
      </div>

      {/* Keyboard hint (visible on focus) */}
      <p className="text-[10px] text-mid-gray/60 hidden group-focus-within:block select-none">
        {t("audioPlayer.keyboardShortcuts")}
      </p>
    </div>
  );
};
