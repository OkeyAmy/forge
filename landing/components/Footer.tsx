import { Cpu, Github } from "lucide-react";
import { FORGE_REPO_URL, GITHUB_APP_INSTALL_URL } from "@/lib/links";

export function Footer() {
  return (
    <footer className="py-12 px-4 border-t border-white/5">
      <div className="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-6 text-sm text-muted">
        <div className="flex items-center gap-3">
          <Cpu className="w-4 h-4 text-accent-blue" />
          <span className="font-semibold text-white">Forge</span>
          <span className="px-2 py-0.5 rounded-full border border-white/10 text-xs">MIT License</span>
        </div>

        <div className="flex items-center gap-6 text-xs">
          <a
            href={GITHUB_APP_INSTALL_URL}
            className="hover:text-white transition-colors"
          >
            Install App
          </a>
          <a
            href="#setup"
            className="hover:text-white transition-colors"
          >
            Workflow
          </a>
        </div>

        <div className="flex items-center gap-4">
          <a
            href={FORGE_REPO_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1.5 hover:text-white transition-colors"
          >
            <Github className="w-4 h-4" />
            GitHub
          </a>
          <span>© {new Date().getFullYear()} Forge</span>
        </div>
      </div>
    </footer>
  );
}
