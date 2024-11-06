local items = {
    {rarity = 3, name = "a"},
    {rarity = 7, name = "b"},
    {rarity = 15, name = "c"},
    {rarity = 35, name = "d"},
    {rarity = 40, name = "e"},
}

function getrandom()
    local rollValue = math.random(0, 100)
    local currentValue = 0
    for _, item in ipairs(items) do
        currentValue = currentValue + item.rarity
        if rollValue <= currentValue then
            return item
        end
    end
end

local t = {};
for i=0, 1000000 do
    local n = getrandom().name;
    t[n] = t[n] == nil and 1 or t[n] + 1
end
for k, v in pairs(t) do
    t[k] = math.round(t[k] / 10000)
end

print(t)