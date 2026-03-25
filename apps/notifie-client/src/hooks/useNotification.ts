import { useCallback } from 'react';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

export function useNotification() {
  const notify = useCallback(async (title: string, body: string) => {
    let permitted = await isPermissionGranted();
    if (!permitted) {
      const permission = await requestPermission();
      permitted = permission === 'granted';
    }

    if (permitted) {
      sendNotification({ title, body });
    }
  }, []);

  return { notify };
}