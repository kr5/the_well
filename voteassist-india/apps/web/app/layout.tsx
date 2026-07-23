import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "VoteAssist India — Voter guidance, not a government portal",
  description:
    "An independent, non-official guide that helps eligible Indian citizens figure out what they need to do, then deep-links to the official Election Commission of India (ECI) services.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
