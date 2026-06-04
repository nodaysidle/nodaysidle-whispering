import type { ReactNode } from "react";
import { Component } from "react";

type ErrorBoundaryProps = { children: ReactNode };
type ErrorBoundaryState = { error: Error | null };

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  render() {
    if (!this.state.error) return this.props.children;

    return (
      <main className="app-shell">
        <section className="error-panel neumorphic-raised">
          <span className="vault-pill">Recovery mode</span>
          <h1>Whispering hit a UI error.</h1>
          <p>Restart the app. Your saved transcript vault is local and preserved.</p>
          <code>{this.state.error.message}</code>
        </section>
      </main>
    );
  }
}
