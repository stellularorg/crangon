wget https://codeberg.org/api/packages/hkau/npm/fusion/-/1.0.11/fusion-1.0.11.tgz -O fusion.tgz
mv ./fusion.tgz ./static/fusion.tgz

cd static
tar -xzf fusion.tgz

mv package/src/css ./css
sed -i -e 's/\/utility.css/\/static\/css\/utility.css/' ./css/fusion.css

rm -r package
rm ./fusion.tgz

cd ../
