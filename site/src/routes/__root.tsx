import { createRootRoute, Outlet } from "@tanstack/react-router";
import { Header } from "@/components/Header";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <div className="min-h-screen bg-background">
      <Header />
      <main className="mx-auto w-full max-w-6xl px-6">
        <Outlet />
      </main>

      {/* Footer */}
      <footer className="border-t border-border mt-24">
        <div className="mx-auto max-w-6xl px-6 py-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <img
                src="https://raw.githubusercontent.com/lassejlv/termy/refs/heads/main/assets/termy_icon.png"
                alt="Termy"
                className="h-5 w-5 rounded"
              />
              <span>Termy</span>
              <span className="text-muted-foreground/50">·</span>
              <span>Open source terminal emulator</span>
            </div>
            <div className="flex items-center gap-6 text-sm text-muted-foreground">
              <a
                href="https://github.com/lassejlv/termy"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition-colors"
              >
                GitHub
              </a>
              <a
                href="https://github.com/lassejlv/termy/releases"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition-colors"
              >
                Releases
              </a>
              <a
                href="https://github.com/lassejlv/termy/issues"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition-colors"
              >
                Issues
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
