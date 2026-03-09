import { useCallback, useMemo } from 'react';
import { MantineProvider, Notification, Loader, Center } from '@mantine/core';
import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';
import './index.css';
import { Header } from './components/Header';
import { ConfigPanel } from './components/ConfigPanel';
import { LayerToolbar } from './components/LayerToolbar';
import { MapViewer } from './components/MapViewer';
import { useMapData } from './hooks/useMapData';
import { useMapStore } from './store/mapStore';
import { getT } from './i18n';

export function App() {
  const store = useMapStore();
  const { mapData, isLoading, error, generationTimeMs, regenerate, usingWasm } = useMapData(store.config);
  const t = getT(store.language);

  const isDark = store.colorScheme === 'dark';

  const polygonCount = useMemo(() => {
    if (!mapData) return 0;
    return mapData.contour.length + mapData.river.length + mapData.territory.length;
  }, [mapData]);

  const handleGenerate = useCallback(async () => {
    store.setIsGenerating(true);
    try {
      await regenerate();
    } finally {
      store.setIsGenerating(false);
    }
  }, [regenerate, store]);

  return (
    <MantineProvider
      defaultColorScheme={store.colorScheme}
      theme={{
        primaryColor: 'blue',
        fontFamily: 'system-ui, -apple-system, sans-serif',
      }}
    >
      <div className={`flex flex-col h-screen overflow-hidden ${isDark ? 'dark' : ''}`}>
        <Header
          language={store.language}
          colorScheme={store.colorScheme}
          seed={store.config.seed}
          generationTimeMs={generationTimeMs}
          polygonCount={polygonCount}
          usingWasm={usingWasm}
          onToggleColorScheme={store.toggleColorScheme}
          onToggleLanguage={store.toggleLanguage}
        />

        <div className="flex flex-1 overflow-hidden">
          <ConfigPanel
            config={store.config}
            onConfigChange={store.setConfig}
            onGenerate={handleGenerate}
            isGenerating={store.isGenerating || isLoading}
            language={store.language}
          />

          <main className="flex-1 relative overflow-hidden flex flex-col">
            {isLoading ? (
              <Center className="flex-1">
                <Loader size="xl" />
              </Center>
            ) : error ? (
              <Center className="flex-1">
                <Notification color="red" title={t.errors.loadFailed} withCloseButton={false}>
                  {error}
                </Notification>
              </Center>
            ) : (
              <MapViewer
                mapData={mapData}
                layers={store.layers}
                colorScheme={store.colorScheme}
              />
            )}
          </main>
        </div>

        <LayerToolbar
          layers={store.layers}
          onToggleLayer={store.toggleLayer}
          language={store.language}
        />
      </div>
    </MantineProvider>
  );
}

export default App;
