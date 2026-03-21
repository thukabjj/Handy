# Wake Command Audio Fixtures

Generated on 2026-03-08 as 16kHz mono PCM WAV files for manual wake-word testing.

Files:
- `wake_up_command.wav` - "Hey Handy, wake up"
- `wake_up_start_listening.wav` - "Hey Handy, start active listening"
- `wake_up_question.wav` - "Hey Handy, what is on my calendar"
- `non_wake_control.wav` - negative control sample without wake phrase

Quick playback (macOS):
```bash
afplay tests/audio/wake_up_command.wav
```

Batch playback:
```bash
for f in tests/audio/*.wav; do echo "==> $f"; afplay "$f"; done
```
