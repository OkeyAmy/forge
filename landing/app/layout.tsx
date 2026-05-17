import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Forge — GitHub App AI Software Engineer",
  description:
    "Install Forge on a GitHub repository, tag an issue, approve a plan, and let an AI software engineer work in E2B and open a pull request.",
  openGraph: {
    title: "Forge — GitHub App AI Software Engineer",
    description:
      "A GitHub App that turns approved issues into E2B-backed pull requests.",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="bg-background font-sans antialiased">{children}</body>
    </html>
  );
}
