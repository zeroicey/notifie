import { useCallback } from 'react';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

export function useNotification() {
  const notify = useCallback(async (title: string, body: string) => {
    console.log('[Notification] Requesting permission...');
    let permitted = await isPermissionGranted();
    console.log('[Notification] Permission granted:', permitted);

    if (!permitted) {
      const permission = await requestPermission();
      console.log('[Notification] Request permission result:', permission);
      permitted = permission === 'granted';
    }

    if (permitted) {
      console.log('[Notification] Sending notification:', title, body);
      sendNotification({ title, body });
    } else {
      console.error('[Notification] Permission denied!');
    }
  }, []);

  return { notify };
}