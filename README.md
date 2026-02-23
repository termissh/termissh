# TermiSSH

Terminal tabanlı SSH bağlantı yöneticisi.

## Kurulum

```bash
cargo build --release
```

## Yapılandırma

API senkronizasyonu için `.env` dosyası oluşturun:

```bash
cp .env.example .env
```

`.env` dosyasını düzenleyin:

```env
API_URL=https://termissh.org
```

API Key'i uygulama içinden ayarlayın (s tuşuna basın).

API yapılandırması opsiyoneldir. Eğer `.env` dosyası yoksa veya API bilgileri girilmemişse, uygulama sadece yerel mod ile çalışır.

## Kullanım

```bash
cargo run
```

### Kısayollar

- `↑/↓`: Sunucu seçimi
- `Enter`: Seçili sunucuya bağlan
- `n`: Yeni sunucu ekle
- `e`: Sunucu düzenle
- `d`: Sunucu sil
- `s`: API Key ayarla
- `q`: Çıkış

### SSH Oturumu İçinde

- `:p` → `pwd`
- `:ls` → `ls -la`
- `:top` → `htop`
- `:u` → `uptime`
- `:d` → `docker ps`
- `:dc` → `docker compose up -d`
- `:q` → `exit`

## API Senkronizasyonu

Eğer `.env` dosyasında `API_URL` tanımlı ve uygulama içinden API Key ayarlanmışsa:

- Uygulama başladığında sunucular API'den çekilir
- Yeni eklenen sunucular API'ye kaydedilir
- Düzenlenen sunucular API'de güncellenir
- Silinen sunucular API'den de silinir

API Key'i ayarlamak için uygulamayı açın ve `s` tuşuna basın.

API bağlantısı başarısız olursa, uygulama yerel config dosyasını kullanır.
