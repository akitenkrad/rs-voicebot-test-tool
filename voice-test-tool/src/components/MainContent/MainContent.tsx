import { WaveformDisplay } from "./WaveformDisplay";
import { PlaybackControls } from "./PlaybackControls";

export function MainContent() {
  return (
    <main className="flex flex-1 flex-col gap-4 overflow-hidden p-4">
      <WaveformDisplay />
      <PlaybackControls />
    </main>
  );
}
