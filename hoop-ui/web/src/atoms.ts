import { atom } from 'jotai';

export interface Conversation {
  id: string;
  title: string;
  messages: Array<{
    id: string;
    role: 'user' | 'assistant';
    content: string;
    timestamp: number;
  }>;
  createdAt: number;
  updatedAt: number;
}

export interface Project {
  name: string;
  path: string;
  activeBeads: number;
  workers: number;
}

export interface Stitch {
  id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
}

export const conversationsAtom = atom<Conversation[]>([]);
export const projectsAtom = atom<Project[]>([]);
export const stitchesAtom = atom<Stitch[]>([]);
