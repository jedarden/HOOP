import { useAtom } from 'jotai';
import { conversationsAtom, projectsAtom, stitchesAtom } from './atoms';

export default function App() {
  const [conversations] = useAtom(conversationsAtom);
  const [projects] = useAtom(projectsAtom);
  const [stitches] = useAtom(stitchesAtom);

  return (
    <div className="app">
      <header>
        <h1>HOOP</h1>
        <p>The operator's pane of glass and conversational handle.</p>
      </header>
      <main>
        <section>
          <h2>Conversations</h2>
          <pre>{JSON.stringify(conversations, null, 2)}</pre>
        </section>
        <section>
          <h2>Projects</h2>
          <pre>{JSON.stringify(projects, null, 2)}</pre>
        </section>
        <section>
          <h2>Stitches</h2>
          <pre>{JSON.stringify(stitches, null, 2)}</pre>
        </section>
      </main>
    </div>
  );
}
