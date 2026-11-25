# Pemeriksaan VPS sebelum pemasangan Zevrigo Node (aplikasi Android, Q199a). Dijalankan aplikasi lewat SSH dengan `sh -s`
# sebagai pengguna SSH, tanpa sudo dan tanpa mengubah apa pun. Keluaran: baris kunci=nilai.
echo "arsitektur=$(uname -m)"
if [ -r /etc/os-release ]; then
  . /etc/os-release
  echo "os=${ID:-}"
  echo "versi_os=${VERSION_ID:-}"
  echo "nama_os=${PRETTY_NAME:-}"
fi
if [ -d /run/systemd/system ]; then echo "systemd=ya"; fi
if [ "$(id -u)" = 0 ]; then echo "root=ya"; fi
if command -v sudo > /dev/null 2>&1; then echo "sudo=ya"; fi
if command -v curl > /dev/null 2>&1 || command -v wget > /dev/null 2>&1; then echo "unduh=ya"; fi
if [ -s /etc/ssl/certs/ca-certificates.crt ]; then echo "ca=ya"; fi
if [ -e /usr/local/sbin/demo-zevrigo-kendali ] || [ -e /etc/systemd/system/demo-zevrigo-node.service ]; then echo "terpasang=ya"; fi
echo "disk_bebas_mb=$(df -Pm / | awk 'NR==2{print $4}')"
echo "ram_mb=$(awk '/^MemTotal:/{print int($2/1024)}' /proc/meminfo)"
exit 0
