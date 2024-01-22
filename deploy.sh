
SPACER="======================================"
EG="🔷"

cd /bots/neptun/

OLD_COMMIT=$(git rev-parse HEAD)

echo "$EG update the source"
git pull
echo $SPACER

NEW_COMMIT=$(git rev-parse HEAD)

function check_diff {
    local file_has_changed=$(git diff --name-only $OLD_COMMIT...$NEW_COMMIT --exit-code $1)
    if [ -z "$file_has_changed" ]; then
        return 1
    else
        return 0
    fi
}

if check_diff "neptun.service"; then
    echo "$EG reload the service"
    cp neptun.service /etc/systemd/system/ --force
    systemctl daemon-reload
    echo $SPACER
fi

cargo build

systemctl restart neptun
systemctl status neptun

echo "Deploy is Done! ✅"

