echo "Downloading GTFS data"

rm -r KRK/
mkdir -p KRK/A KRK/T KRK/M

echo "Downloading KRK"
curl https://gtfs.ztp.krakow.pl/GTFS_KRK_A.zip > A.zip
curl https://gtfs.ztp.krakow.pl/GTFS_KRK_T.zip > T.zip
curl https://gtfs.ztp.krakow.pl/GTFS_KRK_M.zip > M.zip

unzip A.zip -d KRK/A/
unzip T.zip -d KRK/T/
unzip M.zip -d KRK/M/
