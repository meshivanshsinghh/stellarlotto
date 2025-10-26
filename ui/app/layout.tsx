import type { Metadata } from "next";
import { Creepster, Geist } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const creepster = Creepster({
  weight: "400",
  variable: "--font-creepster",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "StellarLotto 🎃 No-Loss Halloween Lottery",
  description: "Win crypto prizes with ZERO risk! Built on Stellar with Blend Protocol yield. Join the spooky lottery now!",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${creepster.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}